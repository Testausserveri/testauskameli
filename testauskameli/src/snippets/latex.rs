//! A Latex service
use crate::utils;
use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};
use anyhow::Result;
use async_trait::async_trait;
use either::Either;
use tracing::*;

/// The service for Latex is a unit struct
pub struct Latex;

#[async_trait]
impl MrSnippet for Latex {
    fn dependencies(&self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "latex"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let latex = content
            .trim()
            .to_string()
            .replace("$", "")
            .replace("latex ", "");

        let latex = if content.starts_with("```latex") {
            content
                .trim()
                .to_string()
                .replace("$", "")
                .replace("```latex", "")
                .replace("```", "")
        } else {
            return Either::Right(Mismatch::Continue);
        };

        Either::Left(Runner::new("latex", "latex", || {
            Box::pin(async move {
                info!("Got latex: {}", latex);
                let url = format!(
                    "https://latex.codecogs.com/png.latex?\\dpi{{300}} {}",
                    latex
                );

                let resp = reqwest::get(&url).await.unwrap();

                if !resp.status().is_success() {
                    info!("Failed to get latex image");
                    let stderr = format!("Failed to get image, status: {}", resp.status());
                    Ok(RunnerOutput::WithError("Failed".to_string(), stderr))
                } else {
                    let data = resp.bytes().await.unwrap();

                    let path = utils::rand_path_with_extension(".png");
                    let mut img = image::load_from_memory(&data).unwrap();
                    img.invert(); // Image from codecogs has black font so it is inverted
                    img.save(&path).unwrap();

                    Ok(RunnerOutput::WithFiles("".into(), vec![path.into()], true))
                }
            })
        }))
    }
}
