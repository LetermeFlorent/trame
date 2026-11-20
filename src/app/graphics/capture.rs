#[cfg(feature = "capture")]
mod inner {
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::BufWriter;
    use std::path::PathBuf;

    use trame::log_error;
    use trame::render::{Gpu, ReadbackError, read_rgba};

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
            if gpu.is_lost() {
                return;
            }
            if let Err(error) = self.write(gpu, texture) {
                log_error!("the frame capture failed: {error}");
            }
        }

        fn write(&self, gpu: &Gpu, texture: &wgpu::Texture) -> Result<(), Box<dyn Error>> {
            if gpu.is_lost() {
                return Err(ReadbackError::Lost.into());
            }
            let (width, height) = (texture.width(), texture.height());
            let mut pixels = read_rgba(gpu, texture)?;
            for pixel in pixels.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
                pixel[ALPHA] = OPAQUE;
            }
            let partial = self.directory.join(PARTIAL);
            let mut encoder =
                png::Encoder::new(BufWriter::new(File::create(&partial)?), width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(&pixels)?;
            fs::rename(partial, self.directory.join(LATEST))?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod render_tests {
        use super::*;
        use trame::render::{OFFSCREEN_FORMAT, ReadbackError};

        #[test]
        fn capture_write_on_lost_device_returns_lost_error_without_writing_files() {
            let dir = std::env::temp_dir().join(format!("trame-cap-{}", std::process::id()));
            fs::create_dir_all(&dir).expect("temp dir");
            let gpu = Gpu::headless().expect("headless gpu");
            let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("capture"),
                size: wgpu::Extent3d {
                    width: 2,
                    height: 2,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            gpu.device.destroy();
            let _ = gpu.device.poll(wgpu::PollType::Poll);
            assert!(gpu.is_lost());
            let capture = Capture {
                directory: dir.clone(),
            };
            let err = capture.write(&gpu, &texture).expect_err("lost readback");
            assert!(
                err.downcast_ref::<ReadbackError>()
                    .is_some_and(|e| { matches!(e, ReadbackError::Lost) })
            );
            capture.save(&gpu, &texture);
            assert!(!dir.join(LATEST).exists());
            assert!(!dir.join(PARTIAL).exists());
            let _ = fs::remove_dir_all(dir);
        }
    }
}

#[cfg(feature = "capture")]
pub use inner::Capture;

#[cfg(not(feature = "capture"))]
pub struct Capture;

#[cfg(not(feature = "capture"))]
impl Capture {
    pub fn from_env() -> Option<Self> {
        None
    }

    pub fn save(&self, _gpu: &trame::render::Gpu, _texture: &wgpu::Texture) {}
}
