//! A testing MrSnippet that just echos text back
use anyhow::Result;
use async_trait::async_trait;
use either::Either;
use tracing::*;

use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};

/// Echo handler, simply returns the text of the message it received in reply
pub struct Echo;

#[async_trait]
impl MrSnippet for Echo {
    fn dependencies(&self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "echo"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let text = if let Some(start) = content.find("echo ") {
            content[start + 5..].to_string()
        } else {
            return Either::Right(Mismatch::Continue);
        };

        Either::Left(Runner::new("echo", "test", || {
            info!("{} (echo)", text);
            Box::pin(async move { Ok(RunnerOutput::Output(text)) })
        }))
    }
}
