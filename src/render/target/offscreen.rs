use std::sync::mpsc;

use super::error::ReadbackError;
use crate::render::gpu::Gpu;
use crate::render::renderer::Renderer;
use crate::render::scene::Scene;
use crate::theme::Rgba;

pub const OFFSCREEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

const BYTES_PER_PIXEL: u32 = 4;

pub fn render_to_rgba(
    gpu: &Gpu,
    renderer: &mut Renderer,
    scene: &Scene,
    clear: Rgba,
) -> Result<Vec<u8>, ReadbackError> {
    render_in_format(gpu, renderer, scene, clear, OFFSCREEN_FORMAT)
}

pub fn render_in_format(
    gpu: &Gpu,
    renderer: &mut Renderer,
    scene: &Scene,
    clear: Rgba,
    format: wgpu::TextureFormat,
) -> Result<Vec<u8>, ReadbackError> {
    let width = scene.width().max(1.0) as u32;
    let height = scene.height().max(1.0) as u32;
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    renderer.render(gpu, &view, scene, clear);
    let mut pixels = read_texture(gpu, &texture)?;
    into_rgba(&mut pixels, format);
    Ok(pixels)
}

pub fn into_rgba(pixels: &mut [u8], format: wgpu::TextureFormat) {
    if matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    ) {
        pixels
            .chunks_exact_mut(4)
            .for_each(|pixel| pixel.swap(0, 2));
    }
}

pub fn read_texture(gpu: &Gpu, texture: &wgpu::Texture) -> Result<Vec<u8>, ReadbackError> {
    let (width, height) = (texture.width(), texture.height());
    let row = width * BYTES_PER_PIXEL;
    let padded = row.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let buffer = copy_to_buffer(gpu, texture, size, padded);
    let slice = buffer.slice(..);
    map_and_wait(gpu, &slice)?;
    let mut pixels = Vec::with_capacity((row * height) as usize);
    {
        let mapped = slice.get_mapped_range().map_err(ReadbackError::Range)?;
        for line in mapped.chunks(padded as usize) {
            pixels.extend_from_slice(&line[..row as usize]);
        }
    }
    buffer.unmap();
    Ok(pixels)
}

fn copy_to_buffer(
    gpu: &Gpu,
    texture: &wgpu::Texture,
    size: wgpu::Extent3d,
    padded: u32,
) -> wgpu::Buffer {
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(padded * size.height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("readback"),
        });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(size.height),
            },
        },
        size,
    );
    gpu.queue.submit([encoder.finish()]);
    buffer
}

fn map_and_wait(gpu: &Gpu, slice: &wgpu::BufferSlice) -> Result<(), ReadbackError> {
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(ReadbackError::Poll)?;
    receiver
        .try_recv()
        .unwrap_or(Err(wgpu::BufferAsyncError))
        .map_err(ReadbackError::Map)
}
