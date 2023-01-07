//! An Idris service
//! Requires Idris2 to run
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use either::Either;
use tracing::*;

use crate::cmd::Command;
use crate::utils;
use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};

/// The service for Idris needs to data so far, and so it is a unit struct
pub struct Idris;

#[async_trait]
impl MrSnippet for Idris {
    fn dependencies(&self) -> Result<()> {
        utils::needed_programs(&["idris2", "idris-runner", "chez"])
    }

    fn name(&self) -> &'static str {
        "idris"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let start = if content.contains("```idris") {
            content.find("```idris").expect("BUG: impossible") + "```idris".len()
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

        Either::Left(Runner::new("idris", "idris", || {
            Box::pin(async move {
                // Files is a deletion guard, keep it or your mom gay
                // Once it is dropped, all temporary files created for this runner will be deleted
                // If you bind it to '_', they will be deleted before the runner even starts
                // due to the ignore pattern implying an early dropped,
                // this doesn't happen with "intentionally unused pattern", ie. _ident
                let (output, _files) =
                    Command::new("idris-runner").run_with_content(code.as_bytes(), Some("hs"));

                info!("Run idris");

                let output = output.await?;

                let stdout = String::from_utf8(output.stdout).unwrap();
                if output.status.success() {
                    info!("Idris finished with success");
                    Ok(RunnerOutput::Output(stdout))
                } else {
                    info!("Idris finished with error");
                    let stderr = String::from_utf8(output.stderr).unwrap();
                    Ok(RunnerOutput::WithError(stdout, stderr))
                }
            })
        }))
    }
}
