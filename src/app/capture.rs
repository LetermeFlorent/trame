use std::error::Error;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

use trame::render::{Gpu, read_texture};

const DIRECTORY_VARIABLE: &str = "TRAME_CAPTURE";
const LATEST: &str = "latest.png";
const PARTIAL: &str = "latest.part";
const OPAQUE: u8 = 255;

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
            eprintln!("trame: the frame capture failed: {error}");
        }
    }

    fn write(&self, gpu: &Gpu, texture: &wgpu::Texture) -> Result<(), Box<dyn Error>> {
        let (width, height) = (texture.width(), texture.height());
        let mut pixels = read_texture(gpu, texture, width, height);
        let swap = matches!(
            texture.format(),
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );
        for pixel in pixels.chunks_exact_mut(4) {
            if swap {
                pixel.swap(0, 2);
            }
            pixel[3] = OPAQUE;
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
