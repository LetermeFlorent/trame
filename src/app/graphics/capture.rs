use std::error::Error;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

use trame::log_error;
use trame::render::{Gpu, read_rgba};

const DIRECTORY_VARIABLE: &str = "TRAME_CAPTURE";
const LATEST: &str = "latest.png";
const PARTIAL: &str = "latest.part";
const OPAQUE: u8 = u8::MAX;
const BYTES_PER_PIXEL: usize = 4;
const ALPHA: usize = 3;

pub struct Capture {
    directory: PathBuf,
}

impl Capture {
    pub fn from_env() -> Option<Self> {
        let directory = PathBuf::from(std::env::var_os(DIRECTORY_VARIABLE)?);
        Some(Self { directory })
    }

    pub fn save(&self, gpu: &Gpu, texture: &wgpu::Texture) {
        if let Err(error) = self.write(gpu, texture) {
            log_error!("the frame capture failed: {error}");
        }
    }

    fn write(&self, gpu: &Gpu, texture: &wgpu::Texture) -> Result<(), Box<dyn Error>> {
        let (width, height) = (texture.width(), texture.height());
        let mut pixels = read_rgba(gpu, texture)?;
        for pixel in pixels.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
            pixel[ALPHA] = OPAQUE;
        }
        let partial = self.directory.join(PARTIAL);
        let mut encoder = png::Encoder::new(BufWriter::new(File::create(&partial)?), width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&pixels)?;
        fs::rename(partial, self.directory.join(LATEST))?;
        Ok(())
    }
}
