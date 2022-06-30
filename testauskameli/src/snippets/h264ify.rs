//! A service for video recoding aimed at making content discord-previewable

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use either::Either;
use reqwest;
use std::fs::File;

use crate::utils;
use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};

/// H264ify service converts videos from different codecs to h264
/// so that previewing them works on official discord clients
pub struct H264ify;

#[async_trait]
impl MrSnippet for H264ify {
    fn dependencies(&self) -> Result<()> {
        utils::needed_programs(&["ffmpeg"])
    }

    fn name(&self) -> &'static str {
        "h264ify"
    }

    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let start = if content.contains("```h264ify") {
            content.find("```h264ify").expect("BUG: impossible") + "```h264ify".len()
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

        let urls: Vec<String> = content[start..end]
            .split('\n')
            .map(|x| x.to_string())
            .collect();
        let mut files = Vec::new();

        Either::Left(Runner::new("h264ify", "h264ify", || {
            // FIXME: Detect wrong usage
            // NOTE: If two videos have the same filename the program works as intended but
            // an error occurs on post-runner cleanup while tryng to delete and already deleted file
            Box::pin(async move {
                for url in &urls {
                    let filename = if let Some(idx) = url.rfind('/') {
                        &url[idx + 1..]
                    } else {
                        url
                    };

                    let mut original_video = File::create(&filename).unwrap();
                    let response = reqwest::get(url).await.unwrap();
                    let mut content = std::io::Cursor::new(response.bytes().await.unwrap());
                    std::io::copy(&mut content, &mut original_video).unwrap();

                    std::process::Command::new("ffmpeg")
                        .args(&[
                            "-i",
                            filename,
                            "-vcodec",
                            "libx264",
                            "-vprofile",
                            "baseline",
                            "-level",
                            "3.0",
                            "-pix_fmt",
                            "yuv420p",
                            &format!("{}-fixed.mp4", &filename),
                        ])
                        .output()
                        .unwrap();

                    std::fs::remove_file(&filename).unwrap();
                    files.push(format!("{}-fixed.mp4", &filename).into());
                }

                Ok(RunnerOutput::WithFiles("".into(), files, true))
            })
        }))
    }
}
