//! A library for creating bots the handsome way.
//!
//! ***AGNOSTICALLY***
#![feature(async_closure)]
#![allow(clippy::new_without_default)]
#![deny(missing_docs)]

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

/// A type indicating to the executor how to proceed in case of mismatch
///
/// This type is returned by [`MrSnippet::try_or_continue`] in the right branch,
/// and it indicates that this input will not produce correct output for this
/// snippet handler.
///
/// This can be due to two reasons:
/// - The input matches the handler, but is invalid (ie. malformed syntax)
/// - The input does not match this handler, and the search should continue
///
/// The two variants of this enum illustrate both reasons.
#[derive(Debug)]
pub enum Mismatch {
    /// Continue search
    Continue,
    /// You are doing this incorrectly.
    /// Executors should try to send a message with complaints
    WrongUsage(Error),
}

/// The output of a runner dispatch
///
/// A [`MrSnippet`] might produce a wide variety of output,
/// for actual code runners, this can be some text output,
/// errors, or maybe nothing, who knows.
///
/// Other handlers might produce files.
///
/// This enum also contains a variant for `WrongUsage`, this is
/// so that the executor can feed it into the [`Executor::send`] method.
/// You shouldn't make this one directly
#[derive(Debug, Clone)]
pub enum RunnerOutput {
    /// No meaningful output was generated
    None,
    /// Use this for stdout
    Output(String),
    /// If there were also some errors. First member of tuple is output, second error (stderr)
    WithError(String, String),
    /// stdout, and a vector of paths that should be attached to the response
    WithFiles(String, Vec<PathBuf>),
    /// Should be only emitted by the [`Executor`] in case of [`Mismatch::WrongUsage`]
    WrongUsage(String),
}

/// The future returned by the Runner,
///
/// In general, you create them like this:
/// ```rust
/// Runner::new("service", "name", || {
///     Box::pin(async move {
///         Ok(RunnerOutput::None)
///     })
/// })
///
/// ```
pub type RunnerFuture = Pin<Box<dyn Future<Output = Result<RunnerOutput>> + Send + Sync>>;
/// See [`RunnerFuture`]
pub type RunnerDispatcher = Box<dyn FnOnce() -> RunnerFuture + Send + Sync>;

/// A Runner is dispatched for every successful match by [`MrSnippet`]
///
/// It should serve to log and provides means to execute commands or code to produce
/// meaningful output.
pub struct Runner {
    /// Name of the service, this will be something like the programming language concerned
    pub service: String,
    /// Use this if you wanna make a distinction between different usages
    pub name: String,
    future: RunnerDispatcher,
}

impl Runner {
    /// Create a new runner for a service
    ///
    /// ```rust
    /// Runner::new("service", "name", || {
    ///     Box::pin(async move {
    ///         Ok(RunnerOutput::None)
    ///     })
    /// })
    ///
    /// ```
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

    /// Dispatch the runner, generating a future to be awaited on
    pub fn dispatch(self) -> RunnerFuture {
        info!("dispatch {}: {}", self.service, self.name);
        (self.future)()
    }
}

/// Trait for a provided service. Implement for every language or
/// another funny.
///
/// The [`MrSnippet::dependencies`] method should return an anyhow error
/// if the host system does not have the faculties to run this service.
///
/// And [`MrSnippet::name`] should return a reasonably unique yet constant name,
/// which will be used to indentify this snippeter.
#[async_trait]
pub trait MrSnippet {
    /// Fail if dependencies for this service are not met
    fn dependencies(&self) -> Result<()> {
        Ok(())
    }

    /// Readable, constant name of this service
    fn name(&self) -> &'static str {
        "u forgor 💀"
    }

    /// Determine, whether output is valid for you, invalid, or unrelated.
    /// The [`MrSnippet`]s are in a cooperative relationship, it is up to
    /// each service to properly indicate that search should continue.
    ///
    /// If input is valid, produce a runner.
    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch>;
}

/// The heart and the brains of the whole operation
///
/// Implement this trait whenever you are creating a bot
#[async_trait]
pub trait Executor {
    /// A type containing values necessary for properly sending a message back
    type Context: Send + Sync;

    /// Register a new [`MrSnippet`] to this executor. Having this allows writing
    /// services in different crates and enabling them by need and want.
    fn register(&self, snippet_handler: Box<dyn MrSnippet + Send + Sync>);

    /// Return an iterator over enabled services. You can try to have some priority
    /// heuristics, if you like.
    fn iter(
        &self,
    ) -> Box<dyn Iterator<Item = Arc<(dyn MrSnippet + Send + Sync)>> + Send + Sync + '_>;

    /// Send a message back. This method will actually interact with whatever
    /// you are writing the [`Executor`] for
    async fn send(&self, content: RunnerOutput, context: &Self::Context) -> Result<()>;

    /// Run the executor (spawn this in a tokio::task or its analogue).
    ///
    /// To stop this executor, simply drop all senders for this receiver.
    ///
    /// You can also just kill the process, it's fine
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

                            info!("dispatch finnished :)");
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
