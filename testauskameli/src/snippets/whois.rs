//! A Whois service
use anyhow::Result;
use async_trait::async_trait;
use either::Either;
use tracing::*;

use crate::cmd::Command;
use crate::utils;
use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};

/// The service for Whois is a unit struct
pub struct Whois;

#[async_trait]
impl MrSnippet for Whois {
    fn dependencies(&self) -> Result<()> {
        utils::needed_programs(&["whois"])
    }

    fn name(&self) -> &'static str {
        "whois"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let domain = if let Some(start) = content.find("whois ") {
            content[start + 6..].trim().to_string()
        } else {
            return Either::Right(Mismatch::Continue);
        };

        Either::Left(Runner::new("whois", "whois", || {
            Box::pin(async move {
                let output = Command::unlimited("whois").arg(domain).run();

                let output = output.await?;

                let mut stdout = String::from_utf8(output.stdout).unwrap();
                stdout.truncate(1900);

                if output.status.success() {
                    info!("Whois finished succesfully");
                    Ok(RunnerOutput::Output(stdout))
                } else {
                    info!("Whois finished unsuccesfully");
                    let mut stderr = String::from_utf8(output.stderr).unwrap();
                    stderr.truncate(1950);
                    Ok(RunnerOutput::WithError(stdout, stderr))
                }
            })
        }))
    }
}
