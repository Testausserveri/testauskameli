//! A JS service
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use either::Either;
use tracing::*;

use crate::cmd::Command;
use crate::utils;
use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};

/// The service for JS needs no data so far, and so it is a unit struct
pub struct JS;

#[async_trait]
impl MrSnippet for JS {
    fn dependencies(&self) -> Result<()> {
        utils::needed_programs(&["js-runner", "node"])
    }

    fn name(&self) -> &'static str {
        "js"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let start = match (content.contains("```js"), content.contains("```js")) {
            (true, _) => content.find("```js").expect("BUG: impossible") + "```js".len(),
            (false, true) => content.find("```js").expect("BUG: impossible") + "```js".len(),
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

        Either::Left(Runner::new("js", "js", || {
            Box::pin(async move {
                let (output, _files) =
                    Command::unlimited("js-runner").run_with_content(code.as_bytes(), Some("js"));

                info!("Run JS");

                let output = output.await?;

                let mut stdout = String::from_utf8(output.stdout).unwrap();
                stdout.truncate(1900);
                if output.status.success() {
                    info!("JS finished with (great)success");
                    Ok(RunnerOutput::Output(stdout))
                } else {
                    info!("JS finished with error");
                    let mut stderr = String::from_utf8(output.stderr).unwrap();
                    stderr.truncate(1950);
                    Ok(RunnerOutput::WithError(stdout, stderr))
                }
            })
        }))
    }
}
