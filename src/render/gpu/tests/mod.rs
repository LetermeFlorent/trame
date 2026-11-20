mod recovery;

use super::{Gpu, GpuError, headless_instance};
use crate::geometry::Rect;
use crate::render::{Atlas, AtlasKind, OFFSCREEN_FORMAT, Renderer, Scene, render_to_rgba};
use crate::theme::Rgba;

pub(super) fn render_white_glyph(gpu: &Gpu, renderer: &mut Renderer, atlas: &mut Atlas) -> Vec<u8> {
    let region = atlas
        .insert(AtlasKind::Mask, 2, 2, vec![255; 4])
        .expect("glyph fits");
    assert!(!atlas.finish_frame());
    renderer.sync_atlas(gpu, atlas);
    let mut scene = Scene::new(4.0, 4.0);
    scene.glyph(region, 0.0, 0.0, Rgba::new(1.0, 1.0, 1.0, 1.0));
    render_to_rgba(gpu, renderer, &scene, Rgba::new(0.0, 0.0, 0.0, 1.0)).expect("render")
}

#[test]
fn failing_backend_returns_adapter_error() {
    let instance = headless_instance(wgpu::Backends::empty());
    assert!(matches!(
        Gpu::new(instance, None),
        Err(GpuError::Adapter(_))
    ));
}

#[test]
fn software_fallback_adapter_creates_cpu_device_and_renders() {
    let gpu = Gpu::headless_software().expect("software adapter");
    assert_eq!(gpu.adapter_info().device_type, wgpu::DeviceType::Cpu);
    let mut atlas = Atlas::new();
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, &mut atlas);
    let mut scene = Scene::new(4.0, 4.0);
    scene.fill(
        Rect::new(0.0, 0.0, 4.0, 4.0),
        Rgba::new(0.0, 1.0, 0.0, 1.0),
        0.0,
    );
    let pixels = render_to_rgba(&gpu, &mut renderer, &scene, Rgba::new(0.0, 0.0, 0.0, 0.0))
        .expect("software render");
    assert_eq!(&pixels[..4], &[0, 255, 0, 255]);
}

#[test]
fn headless_falls_back_from_empty_backend_to_software_with_logs() {
    let mut logs = Vec::new();
    let gpu = Gpu::headless_with(wgpu::Backends::empty(), &mut |line| {
        logs.push(line.to_owned())
    })
    .expect("software fallback succeeds");
    assert_eq!(gpu.adapter_info().device_type, wgpu::DeviceType::Cpu);
    assert_eq!(logs.len(), 2);
    assert!(logs[0].contains("software adapter"), "{logs:?}");
    assert!(
        logs[1].contains("using the software graphics adapter"),
        "{logs:?}"
    );
}
