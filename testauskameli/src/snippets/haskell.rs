//! A Haskell service
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use either::Either;
use tracing::*;

use crate::cmd::Command;
use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};

/// The service for Haskell needs to data so far, and so it is a unit struct
pub struct Haskell;

#[async_trait]
impl MrSnippet for Haskell {
    fn dependencies(&self) -> Result<()> {
        // TODO:
        //    - check for haskell install
        //    - check for haskell runner script
        //    - idk if anything else will be needed
        //      (would we include s6 tools here?)
        Ok(())
    }

    fn name(&self) -> &'static str {
        "haskell"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let start = match (content.contains("```hs"), content.contains("```haskell")) {
            (true, _) => content.find("```hs").expect("BUG: impossible") + "```hs".len(),
            (false, true) => {
                content.find("```haskell").expect("BUG: impossible") + "```haskell".len()
            }
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

        Either::Left(Runner::new("haskell", "haskell", || {
            Box::pin(async move {
                // Files is a deletion guard, keep it or your mom gay
                // Once it is dropped, all temporary files created for this runner will be deleted
                // If you bind it to '_', they will be deleted before the runner even starts
                // due to the ignore pattern implying an early dropped,
                // this doesn't happen with "intentionally unused pattern", ie. _ident
                let (output, _files) = Command::unlimited("haskell-runner")
                    .run_with_content(code.as_bytes(), Some("hs"));

                info!("Run haskell");

                let output = output.await?;

                if output.status.success() {
                    info!("Haskell finished with success");
                    let mut stdout = String::from_utf8(output.stdout).unwrap();
                    stdout.truncate(1900);
                    Ok(RunnerOutput::Output(stdout))
                } else {
                    info!("Haskell finished with error");
                    let mut stderr = String::from_utf8(output.stderr).unwrap();
                    stderr.truncate(1950);
                    // TODO: there might still be some output, ie. in case of Rust with warnings,
                    //       so it should be included and processed correctly in those cases
                    Ok(RunnerOutput::WithError(String::new(), stderr))
                }
            })
        }))
    }
}
