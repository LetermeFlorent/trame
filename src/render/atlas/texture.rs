use super::{AtlasKind, Upload};

pub(in crate::render) struct AtlasTexture {
    kind: AtlasKind,
    size: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl AtlasTexture {
    pub(in crate::render) fn new(device: &wgpu::Device, kind: AtlasKind, size: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas"),
            size: extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format(kind),
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            kind,
            size,
            texture,
            view,
        }
    }

    pub(in crate::render) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub(in crate::render) fn fit(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: u32,
    ) -> bool {
        if size == self.size {
            return false;
        }
        let resized = Self::new(device, self.kind, size);
        if size < self.size {
            *self = resized;
            return true;
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("atlas grow"),
        });
        encoder.copy_texture_to_texture(
            self.texture.as_image_copy(),
            resized.texture.as_image_copy(),
            extent(self.size),
        );
        queue.submit([encoder.finish()]);
        *self = resized;
        true
    }

    pub(in crate::render) fn write(&self, queue: &wgpu::Queue, upload: &Upload) {
        let region = upload.region;
        if region.width == 0 || region.height == 0 {
            return;
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.x,
                    y: region.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &upload.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(region.width * self.kind.bytes_per_pixel()),
                rows_per_image: Some(region.height),
            },
            wgpu::Extent3d {
                width: region.width,
                height: region.height,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn format(kind: AtlasKind) -> wgpu::TextureFormat {
    match kind {
        AtlasKind::Mask => wgpu::TextureFormat::R8Unorm,
        AtlasKind::Color => wgpu::TextureFormat::Rgba8Unorm,
    }
}

fn extent(size: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    }
}
