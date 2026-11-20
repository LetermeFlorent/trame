use super::gpu::Gpu;
use super::renderer::Renderer;

const BYTES_PER_PIXEL: u32 = 4;

pub(super) struct PictureTexture {
    size: (u32, u32),
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl PictureTexture {
    pub(super) fn empty(device: &wgpu::Device) -> Self {
        Self::new(device, 1, 1)
    }

    pub(super) fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("picture"),
            size: extent(width, height),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            size: (width, height),
            _texture: texture,
            view,
        }
    }

    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub(super) fn write(&self, queue: &wgpu::Queue, pixels: &[u8]) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self._texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.size.0 * BYTES_PER_PIXEL),
                rows_per_image: Some(self.size.1),
            },
            extent(self.size.0, self.size.1),
        );
    }
}

impl Renderer {
    pub fn set_picture(&mut self, gpu: &Gpu, image: Option<(u32, u32, &[u8])>) {
        let max_side = gpu.device.limits().max_texture_dimension_2d;
        let valid = image.filter(|(width, height, pixels)| {
            *width > 0
                && *height > 0
                && *width <= max_side
                && *height <= max_side
                && (*width as usize)
                    .checked_mul(*height as usize)
                    .and_then(|count| count.checked_mul(BYTES_PER_PIXEL as usize))
                    == Some(pixels.len())
        });
        match valid {
            None => {
                if self.bindings.picture.size != (1, 1) {
                    self.bindings.picture = PictureTexture::empty(&gpu.device);
                    self.bind_group = self.bindings.group(&gpu.device);
                } else {
                    self.bindings.picture.write(&gpu.queue, &[0; 4]);
                }
            }
            Some((width, height, pixels)) => {
                if self.bindings.picture.size != (width, height) {
                    self.bindings.picture = PictureTexture::new(&gpu.device, width, height);
                    self.bind_group = self.bindings.group(&gpu.device);
                }
                self.bindings.picture.write(&gpu.queue, pixels);
            }
        }
    }
}

fn extent(width: u32, height: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    }
}
