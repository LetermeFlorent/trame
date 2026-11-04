#[cfg(unix)]
mod atomic;
mod capture;
mod icons;
mod log;
mod open;
mod resample;
#[path = "../support/sandbox.rs"]
mod sandbox;

use std::fs;
use std::path::{Path, PathBuf};

use png::{BitDepth, ColorType, Encoder};
use sandbox::Sandbox;

impl Sandbox {
    fn icons(&self) -> PathBuf {
        self.root.join("icons")
    }

    fn picture(&self, name: &str, width: u32, height: u32, pixels: &[u8]) -> PathBuf {
        let path = self.root.join(name);
        write_png(&path, width, height, pixels);
        path
    }
}

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().expect("a png header");
    writer.write_image_data(pixels).expect("png pixels");
    writer.finish().expect("a finished png");
    fs::write(path, bytes).expect("the picture is written");
}

fn solid(width: u32, height: u32, pixel: [u8; 4]) -> Vec<u8> {
    pixel.repeat((width * height) as usize)
}
