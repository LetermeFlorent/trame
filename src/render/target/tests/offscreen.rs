use super::super::{OFFSCREEN_FORMAT, ReadbackError, read_rgba, render_to_rgba};
use crate::geometry::Rect;
use crate::render::{Atlas, Gpu, Renderer, Scene};
use crate::theme::Rgba;

#[test]
fn read_rgba_on_destroyed_device_returns_lost_error() {
    let gpu = Gpu::headless().expect("headless gpu");
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("lost readback"),
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
    assert!(gpu.poll_lost());
    assert!(matches!(
        read_rgba(&gpu, &texture),
        Err(ReadbackError::Lost)
    ));
}

#[test]
fn render_to_rgba_on_destroyed_device_returns_lost_error() {
    let gpu = Gpu::headless().expect("headless gpu");
    let mut atlas = Atlas::new();
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, &mut atlas);
    let mut scene = Scene::new(4.0, 4.0);
    scene.fill(
        Rect::new(0.0, 0.0, 4.0, 4.0),
        Rgba::new(1.0, 0.0, 0.0, 1.0),
        0.0,
    );
    gpu.device.destroy();
    assert!(gpu.poll_lost());
    let result = render_to_rgba(&gpu, &mut renderer, &scene, Rgba::new(0.0, 0.0, 0.0, 1.0));
    assert!(matches!(result, Err(ReadbackError::Lost)));
}

#[test]
fn render_to_rgba_on_unpolled_destroyed_device_returns_lost() {
    let gpu = Gpu::headless().expect("headless gpu");
    let mut atlas = Atlas::new();
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, &mut atlas);
    let mut scene = Scene::new(4.0, 4.0);
    scene.fill(
        Rect::new(0.0, 0.0, 4.0, 4.0),
        Rgba::new(1.0, 0.0, 0.0, 1.0),
        0.0,
    );
    gpu.device.on_uncaptured_error(std::sync::Arc::new(|_| {}));
    gpu.device.destroy();
    let clear = Rgba::new(0.0, 0.0, 0.0, 1.0);
    let result = render_to_rgba(&gpu, &mut renderer, &scene, clear);
    assert!(matches!(result, Err(ReadbackError::Lost)));
}
