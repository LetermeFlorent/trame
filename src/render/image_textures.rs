use std::collections::HashMap;

use super::gpu::Gpu;
use super::image_bindings::ImageBindings;
use super::image_downscale::{MAX_TEXTURE_SIDE, downscale};
use crate::term::graphics::Image;

const BYTES_PER_PIXEL: u32 = 4;
const IDLE_FRAMES: u64 = 120;

struct Entry {
    _texture: wgpu::Texture,
    group: wgpu::BindGroup,
    used: u64,
}

#[derive(Default)]
pub(super) struct ImageTextures {
    map: HashMap<u64, Entry>,
    frame: u64,
}

impl ImageTextures {
    pub(super) fn sync(&mut self, gpu: &Gpu, bindings: &ImageBindings, uploads: &[Image]) {
        let limit = gpu.device.limits().max_texture_dimension_2d;
        let limit = limit.min(MAX_TEXTURE_SIDE);
        for image in uploads {
            if let Some(entry) = self.map.get_mut(&image.serial) {
                entry.used = self.frame;
                continue;
            }
            let (size, pixels) = downscale(image, limit);
            let texture = create_texture(&gpu.device, &gpu.queue, size, &pixels);
            let group = bindings.group(&gpu.device, &texture);
            let entry = Entry {
                _texture: texture,
                group,
                used: self.frame,
            };
            self.map.insert(image.serial, entry);
        }
    }

    pub(super) fn group(&self, serial: u64) -> Option<&wgpu::BindGroup> {
        self.map.get(&serial).map(|entry| &entry.group)
    }

    pub(super) fn end_frame(&mut self) {
        let frame = self.frame;
        self.map
            .retain(|_, entry| frame.saturating_sub(entry.used) < IDLE_FRAMES);
        self.frame += 1;
    }
}

pub(super) fn create_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    (width, height): (u32, u32),
    pixels: &[u8],
) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * BYTES_PER_PIXEL),
            rows_per_image: Some(height),
        },
        size,
    );
    texture
}
