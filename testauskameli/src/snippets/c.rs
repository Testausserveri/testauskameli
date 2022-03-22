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
        let start = match (content.contains("```c"), content.contains("```c")) {
            (true, _) => content.find("```c").expect("BUG: impossible") + "```c".len(),
            (false, true) => content.find("```c").expect("BUG: impossible") + "```c".len(),
            _ => return Either::Right(Mismatch::Continue),
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
                    Command::unlimited("c-runner").run_with_content(code.as_bytes(), Some("c"));

                info!("Run c");

                let output = output.await?;

                let mut stdout = String::from_utf8(output.stdout).unwrap();
                stdout.truncate(1900);
                if output.status.success() {
                    info!("C finished with (great)success");
                    Ok(RunnerOutput::Output(stdout))
                } else {
                    info!("C finished with error");
                    let mut stderr = String::from_utf8(output.stderr).unwrap();
                    stderr.truncate(1950);
                    Ok(RunnerOutput::WithError(stdout, stderr))
                }
            })
        }))
    }
}
