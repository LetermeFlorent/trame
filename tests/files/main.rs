#[cfg(unix)]
mod atomic;
mod icons;
mod resample;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use png::{BitDepth, ColorType, Encoder};

struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let root = env::temp_dir().join(format!("trame-icons-{}-{name}", process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("the sandbox folder is created");
        Self { root }
    }

    fn icons(&self) -> PathBuf {
        self.root.join("icons")
    }

    fn picture(&self, name: &str, width: u32, height: u32, pixels: &[u8]) -> PathBuf {
        let path = self.root.join(name);
        write_png(&path, width, height, pixels);
        path
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
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
