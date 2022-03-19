#![feature(async_closure)]
#![allow(clippy::new_without_default)]

use std::future::Future;
use std::iter::Iterator;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{Context, Error, Result};
use either::Either;
use flume::Receiver;
use tracing::*;

use async_trait::async_trait;

pub mod cmd;
pub mod snippets;

#[derive(Debug)]
pub enum Mismatch {
    Continue,
    WrongUsage(Error),
}

#[derive(Debug, Clone)]
pub enum RunnerOutput {
    None,
    Output(String),
    WithError(String, String),
    WithFiles(String, Vec<PathBuf>),
    WrongUsage(String),
}

pub type RunnerFuture = Pin<Box<dyn Future<Output = Result<RunnerOutput>> + Send + Sync>>;
pub type RunnerDispatcher = Box<dyn FnOnce() -> RunnerFuture + Send + Sync>;

pub struct Runner {
    pub service: String,
    pub name: String,
    future: RunnerDispatcher,
}

impl Runner {
    pub fn new<F>(service: &str, name: &str, fut: F) -> Self
    where
        F: FnOnce() -> RunnerFuture + Send + Sync + 'static,
    {
        Self {
            service: service.to_string(),
            name: name.to_string(),
            future: Box::new(fut),
        }
    }

    pub fn dispatch(self) -> RunnerFuture {
        info!("dispatch {}: {}", self.service, self.name);
        (self.future)()
    }
}

#[async_trait]
pub trait MrSnippet {
    fn dependencies(&self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "u forgor 💀"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch>;
}

#[async_trait]
pub trait Executor {
    type Context: Send + Sync;

    fn register(&self, snippet_handler: Box<dyn MrSnippet + Send + Sync>);
    fn iter(
        &self,
    ) -> Box<dyn Iterator<Item = Arc<(dyn MrSnippet + Send + Sync)>> + Send + Sync + '_>;
    async fn send(&self, content: RunnerOutput, context: &Self::Context) -> Result<()>;

    async fn run(&self, receiver: Receiver<(String, Self::Context)>) {
        while let Ok((message, context)) = receiver.recv_async().await {
            info!("recv message");

            for handler in self.iter() {
                match handler.try_or_continue(&message).await {
                    Either::Left(runner) => {
                        // gentlemen, I like this syntax, so I am doing a funny on line 1
                        let exec_result = async move || -> Result<()> {
                            info!("match found: {}", handler.name());
                            let output = runner.dispatch().await?;

                            debug!("dispatch finnished :)");
                            self.send(output, &context)
                                .await
                                .context("failed to send message after success")?;

                            Ok(())
                        }()
                        .await;

                        if let Err(e) = exec_result {
                            warn!("dispatcher errored: {}", e);
                        }

                        break;
                    }
                    Either::Right(mismatch) => match mismatch {
                        Mismatch::Continue => continue,
                        Mismatch::WrongUsage(err) => {
                            if let Err(e) = self
                                .send(RunnerOutput::WrongUsage(err.to_string()), &context)
                                .await
                            {
                                error!("failed to send message after failure: {}", e);
                            }
                        }
                    },
                }
            }
        }

        info!("channel closed, goodbye");
    }
}
