use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use tracing::*;

use testauskameli::Executor;
use testauskameli::MrSnippet;
use testauskameli::RunnerOutput;

pub(crate) struct CliExecutor {
    handlers: DashMap<&'static str, Arc<dyn MrSnippet + Send + Sync>>,
}

impl CliExecutor {
    pub(crate) fn new() -> Self {
        Self {
            handlers: DashMap::new(),
        }
    }
}

#[async_trait]
impl Executor for CliExecutor {
    type Context = ();

    fn register(&self, handler: Box<dyn MrSnippet + Send + Sync>) {
        self.handlers.insert(handler.name(), handler.into());
    }

    fn iter(
        &self,
    ) -> Box<dyn Iterator<Item = Arc<(dyn MrSnippet + Send + Sync)>> + Send + Sync + '_> {
        Box::new(self.handlers.iter().map(|v| v.value().clone()))
    }

    async fn send(&self, content: RunnerOutput, _context: &Self::Context) -> Result<()> {
        match content {
            RunnerOutput::Output(text) => {
                info!("output type: Output");
                println!("Output:\n\n{}", text);
            }
            RunnerOutput::WithError(text, error) => {
                info!("output type: WithError");
                println!("Output:\n\n{}\nError:{}", text, error);
            }
            RunnerOutput::WithFiles(text, files, delete) => {
                info!("output type: WithFiles");
                println!("Output:\n\n{}\n", text);
                println!("The following files were also created");
                for file in files {
                    println!("    - {}", file.display());
                }
                println!("File deletion hint: {}", delete);
            }
            RunnerOutput::WrongUsage(error) => {
                info!("output type: WrongUsage");
                println!("Wrong usage: {}", error);
            }
            RunnerOutput::None => {
                info!("output type: None");
            }
        }
        Ok(())
    }
}
