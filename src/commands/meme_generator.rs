use image::io::Reader as ImageReader;
use image::{Pixel, Rgb};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
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
        y: height,
    };

    // TODO: Make better
    let mut text = format!("NO {}?", what.to_uppercase());
    text.truncate(18);
    let x = (18 - text.len()) / 2;
    draw_text_mut(
        &mut img,
        Rgb::from([255u8, 255u8, 255u8]).to_rgba(),
        (x as f32 * scale.y) as u32,
        10,
        scale,
        &font,
        &text,
    );

    img.save(path).unwrap();
    String::from("test.png")
}
