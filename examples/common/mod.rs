use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use trame::color::Rgba;
use trame::render::{Gpu, OFFSCREEN_FORMAT, Renderer, Scene, render_to_rgba};
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};

pub struct Args {
    pub output: PathBuf,
    pub scale: f32,
}

impl Args {
    pub fn parse(usage: &'static str) -> Result<Self, Box<dyn Error>> {
        let mut args = env::args_os().skip(1);
        let output = PathBuf::from(args.next().ok_or(usage)?);
        let scale = args
            .next()
            .and_then(|value| value.to_str()?.parse().ok())
            .unwrap_or(1.0);
        Ok(Self { output, scale })
    }
}

pub struct Studio {
    pub text: TextSystem,
    gpu: Gpu,
    renderer: Renderer,
}

impl Studio {
    pub fn new(font_size: f32) -> Result<Self, Box<dyn Error>> {
        let mut text = TextSystem::new(font_size, DEFAULT_LINE_HEIGHT)?;
        let gpu = Gpu::headless()?;
        let renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, text.atlas_mut());
        Ok(Self {
            text,
            gpu,
            renderer,
        })
    }

    pub fn save(
        &mut self,
        scene: &Scene,
        background: Rgba,
        path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        self.text.end_frame();
        self.renderer.sync_atlas(&self.gpu, self.text.atlas_mut());
        let pixels = render_to_rgba(&self.gpu, &mut self.renderer, scene, background)?;
        write_png(path, scene.width() as u32, scene.height() as u32, &pixels)
    }
}

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(pixels)?;
    Ok(())
}
