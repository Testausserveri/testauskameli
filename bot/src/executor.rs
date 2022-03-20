use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use tracing::*;

use testauskameli::Executor;
use testauskameli::MrSnippet;
use testauskameli::RunnerOutput;

use serenity::model::prelude::*;
use serenity::prelude::*;

pub(crate) struct DiscordExecutor {
    handlers: DashMap<&'static str, Arc<dyn MrSnippet + Send + Sync>>,
}

impl DiscordExecutor {
    pub(crate) fn new() -> Self {
        Self {
            handlers: DashMap::new(),
        }
    }
}

#[async_trait]
impl Executor for DiscordExecutor {
    type Context = (Context, Message);

    fn register(&self, handler: Box<dyn MrSnippet + Send + Sync>) {
        self.handlers.insert(handler.name(), handler.into());
    }

    fn iter(
        &self,
    ) -> Box<dyn Iterator<Item = Arc<(dyn MrSnippet + Send + Sync)>> + Send + Sync + '_> {
        Box::new(self.handlers.iter().map(|v| v.value().clone()))
    }

    async fn send(&self, content: RunnerOutput, context: &Self::Context) -> Result<()> {
        let (context, message) = context;

        match content {
            RunnerOutput::Output(text) => {
                message
                    .reply(&context.http, format!("Output:\n```\n{}\n```", text))
                    .await?;
            }
            RunnerOutput::WithError(output, error) => {
                message
                    .reply(
                        &context.http,
                        format!("Output:\n```\n{}\n```\nError:\n```\n{}\n```", output, error),
                    )
                    .await?;
            }
            RunnerOutput::WithFiles(output, files, delete) => {
                message
                    .channel_id
                    .send_message(&context.http, |m| {
                        for file in &files {
                            m.add_file(file);
                        }
                        m.content(output);
                        m.reference_message(message)
                    })
                    .await?;

                if delete {
                    for file in &files {
                        if let Err(e) = tokio::fs::remove_file(&file).await {
                            error!("failed to delete file {}: {}", file.display(), e);
                        }
                    }
                }
            }
            RunnerOutput::WrongUsage(error) => {
                message
                    .reply(&context.http, format!("Incorrect: {}", error))
                    .await?;
            }
            RunnerOutput::None => (),
        }

        Ok(())
    }
}
