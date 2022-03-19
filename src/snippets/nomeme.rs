//! A default implementation for generating memes with "No bitches?" Megamind
use anyhow::Result;
use async_trait::async_trait;
use either::Either;

use image::io::Reader as ImageReader;
use image::{Pixel, Rgb};
use imageproc::drawing::draw_text_mut;
use regex::Regex;
use rusttype::{Font, Scale};

use std::cmp::min;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::{Mismatch, MrSnippet, Runner, RunnerOutput};

/// No meme handler. Contains a compiled regex because
/// compiling it again for every message is an animalistic practice
pub struct NoMeme {
    regex: Regex,
}

impl NoMeme {
    /// Create a new [`NoMeme`] handler. No Snippet?
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"no\s+(.*)?\?").unwrap(),
        }
    }
}

#[async_trait]
impl MrSnippet for NoMeme {
    fn dependencies(&self) -> Result<()> {
        // TODO:
        //     - check for dependencies if there are any
        //      (would we include s6 tools here?)
        Ok(())
    }

    fn name(&self) -> &'static str {
        "no meme"
    }

    // TODO: make much better
    async fn try_or_continue(&self, content: &str) -> Either<Runner, Mismatch> {
        let text = if let Some(cap) = self.regex.captures_iter(content).next() {
            cap.get(1).unwrap().as_str().to_string()
        } else {
            return Either::Right(Mismatch::Continue);
        };

        Either::Left(Runner::new("no meme", "no meme", || {
            Box::pin(async move {
                let path = Path::new("test.png");
                let mut img = ImageReader::open("img/no.png").unwrap().decode().unwrap();

                let mut font = Vec::new();
                File::open(std::env::var("FONT_PATH").unwrap())
                    .unwrap()
                    .read_to_end(&mut font)
                    .unwrap();
                let font = Font::try_from_vec(font).unwrap();

                let height = 40.0;
                let scale = Scale {
                    x: height * 2.0,
                    y: height * 1.2,
                };

                // TODO: Make better
                let mut text = format!("NO {}?", text.to_uppercase());
                let mut y = 10;
                while !text.is_empty() {
                    let e = min(text.len(), 17);
                    let k: String = text.drain(..e).collect();
                    let x = (17 - k.len()) / 2;
                    draw_text_mut(
                        &mut img,
                        Rgb::from([255u8, 255u8, 255u8]).to_rgba(),
                        (x as f32 * scale.y) as u32,
                        y,
                        scale,
                        &font,
                        &k,
                    );
                    y += scale.y as u32;
                }
                img.save(path).unwrap();
                Ok(RunnerOutput::WithFiles("".into(), vec!["test.png".into()]))
            })
        }))
    }
}
