mod atlas;
mod formats;

use std::sync::Arc;

use trame::render::{Gpu, OFFSCREEN_FORMAT, ReadbackError, read_texture};

const WIDTH: u32 = 3;
const HEIGHT: u32 = 2;
const BYTES_PER_PIXEL: u32 = 4;

fn extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: WIDTH,
        height: HEIGHT,
        depth_or_array_layers: 1,
    }
}

fn texture(gpu: &Gpu) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("readback test"),
        size: extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: OFFSCREEN_FORMAT,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn pattern() -> Vec<u8> {
    (0..WIDTH * HEIGHT * BYTES_PER_PIXEL)
        .map(|index| index as u8)
        .collect()
}

#[test]
fn readback_returns_each_row_without_its_padding() {
    let gpu = Gpu::headless().expect("a graphics adapter");
    let texture = texture(&gpu);
    gpu.queue.write_texture(
        texture.as_image_copy(),
        &pattern(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(WIDTH * BYTES_PER_PIXEL),
            rows_per_image: Some(HEIGHT),
        },
        extent(),
    );
    let pixels = read_texture(&gpu, &texture).expect("the pixels are read");
    assert_eq!(pixels, pattern());
}

#[test]
fn a_lost_device_is_reported_instead_of_empty_pixels() {
    let gpu = Gpu::headless().expect("a graphics adapter");
    let texture = texture(&gpu);
    gpu.device.on_uncaptured_error(Arc::new(|_| {}));
    gpu.device.destroy();
    let read = read_texture(&gpu, &texture);
    assert!(matches!(read, Err(ReadbackError::Map(_))), "{read:?}");
}
