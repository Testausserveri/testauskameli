use image::io::Reader as ImageReader;
use image::{Pixel, Rgb};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::cmp::min;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn no(what: &str) -> String {
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
    let mut text = format!("NO {}?", what.to_uppercase());
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
    String::from("test.png")
}
