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
            RunnerOutput::Output(mut text) => {
                text.truncate(1900);
                message
                    .reply(
                        &context.http,
                        format!("Output:\n```\n{}\n```", text.replace("`", "\\`")),
                    )
                    .await?;
            }
            RunnerOutput::WithError(mut output, mut error) => {
                message
                    .reply(
                        &context.http,
                        if output.trim().is_empty() {
                            error.truncate(1900);
                            format!("Error:\n```\n{}\n\n```", error.replace("`", "\\`"))
                        } else {
                            output.truncate(900);
                            error.truncate(900);
                            format!(
                                "Output:\n```\n{}\n\n```\nError:\n```\n{}\n```",
                                output.replace("`", "\\`"),
                                error.replace("`", "\\`")
                            )
                        },
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
                        if !output.trim().is_empty() {
                            m.content(output.replace("`", "\\`"));
                        }
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
                    .reply(
                        &context.http,
                        format!("Incorrect: {}", error.replace("`", "\\`")),
                    )
                    .await?;
            }
            RunnerOutput::None => (),
        }

        Ok(())
    }
}
