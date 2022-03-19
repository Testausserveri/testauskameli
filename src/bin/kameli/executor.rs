use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;

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

    async fn send(&self, _content: RunnerOutput, _context: &Self::Context) -> Result<()> {
        println!("This is where I'd put my sender, if I had one");
        Ok(())
    }
}
