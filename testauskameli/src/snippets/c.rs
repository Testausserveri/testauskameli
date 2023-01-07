//! A C service
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use either::Either;
use tracing::*;

use crate::cmd::Command;
use crate::utils;
use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};

/// The service for C needs no data so far, and so it is a unit struct
pub struct C;

#[async_trait]
impl MrSnippet for C {
    fn dependencies(&self) -> Result<()> {
        utils::needed_programs(&["gcc", "c-runner"])
    }

    fn name(&self) -> &'static str {
        "c"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let start = if content.contains("```c") {
            content.find("```c").expect("BUG: impossible") + "```c".len()
        } else {
            return Either::Right(Mismatch::Continue);
        };

        let end = match content.rfind("```") {
            Some(idx) => idx,
            None => {
                return Either::Right(Mismatch::WrongUsage(anyhow!(
                    "You done fucked up, missing closing backtics"
                )))
            }
        };

        let code = content[start..end].to_string();

        Either::Left(Runner::new("c", "c", || {
            Box::pin(async move {
                let (output, _files) =
                    Command::new("c-runner").run_with_content(code.as_bytes(), Some("c"));

                info!("Run c");

                let output = output.await?;

                let stdout = String::from_utf8(output.stdout).unwrap();
                if output.status.success() {
                    info!("C finished with (great)success");
                    Ok(RunnerOutput::Output(stdout))
                } else {
                    info!("C finished with error");
                    let stderr = String::from_utf8(output.stderr).unwrap();
                    Ok(RunnerOutput::WithError(stdout, stderr))
                }
            })
        }))
    }
}
