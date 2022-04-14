//! A Common Lisp service
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use either::Either;
use tracing::*;

use crate::cmd::Command;
use crate::utils;
use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};

/// The service for Lisp needs no data so far, and so it is a unit struct
pub struct Lisp;

#[async_trait]
impl MrSnippet for Lisp {
    fn dependencies(&self) -> Result<()> {
        utils::needed_programs(&["clisp"])
    }

    fn name(&self) -> &'static str {
        "lisp"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let start = if content.contains("```lisp") {
            content.find("```lisp").expect("BUG: impossible") + "```lisp".len()
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

        Either::Left(Runner::new("lisp", "lisp", || {
            Box::pin(async move {
                // Files is a deletion guard, keep it or your mom gay
                // Once it is dropped, all temporary files created for this runner will be deleted
                // If you bind it to '_', they will be deleted before the runner even starts
                // due to the ignore pattern implying an early dropped,
                // this doesn't happen with "intentionally unused pattern", ie. _ident
                let (output, _files) = Command::unlimited("lisp-runner")
                    .run_with_content(code.as_bytes(), Some("lisp"));

                info!("Run lisp");

                let output = output.await?;

                let mut stdout = String::from_utf8(output.stdout).unwrap();
                stdout.truncate(1900);
                if output.status.success() {
                    info!("Lisp finished with success");
                    Ok(RunnerOutput::Output(stdout))
                } else {
                    info!("Lisp finished with error");
                    let mut stderr = String::from_utf8(output.stderr).unwrap();
                    stderr.truncate(1950);
                    Ok(RunnerOutput::WithError(stdout, stderr))
                }
            })
        }))
    }
}
