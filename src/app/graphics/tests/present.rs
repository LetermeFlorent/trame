use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::super::session::{FrameOutcome, FrameTexture};
use super::open_headless;
use trame::render::{Atlas, Gpu, OFFSCREEN_FORMAT, Renderer, Scene};
use trame::theme::Rgba;

struct TestFrame {
    texture: wgpu::Texture,
    dropped: Arc<AtomicBool>,
}

impl Drop for TestFrame {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}

impl FrameTexture for TestFrame {
    fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}

fn test_frame(gpu: &Gpu) -> (TestFrame, Arc<AtomicBool>) {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("present test"),
        size: wgpu::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: OFFSCREEN_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let dropped = Arc::new(AtomicBool::new(false));
    let frame = TestFrame {
        texture,
        dropped: dropped.clone(),
    };
    (frame, dropped)
}

fn lose_device(gpu: &Gpu) {
    gpu.device.destroy();
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
}

fn run_present(
    prepare: impl FnOnce(&Gpu),
    on_render: impl FnOnce(&Gpu, &mut Renderer, &wgpu::TextureView),
    on_present: impl FnOnce(&Gpu),
) -> (FrameOutcome, bool, bool) {
    let mut atlas = Atlas::new();
    let mut session = open_headless(&mut atlas, false).expect("session");
    prepare(&session.gpu);
    let (frame, dropped) = test_frame(&session.gpu);
    let mut presented = false;
    let outcome = session.present_with(&mut atlas, frame, on_render, |gpu, texture| {
        presented = true;
        on_present(gpu);
        drop(texture);
    });
    (outcome, presented, dropped.load(Ordering::Acquire))
}

#[test]
fn post_render_loss_drops_frame_and_skips_present() {
    let scene = Scene::new(4.0, 4.0);
    let clear = Rgba::new(0.0, 0.0, 0.0, 1.0);
    let (outcome, presented, dropped) = run_present(
        |_| {},
        |gpu, renderer, view| {
            renderer.render(gpu, view, &scene, clear);
            lose_device(gpu);
        },
        |_| {},
    );
    assert!(matches!(outcome, FrameOutcome::Lost));
    assert!(!presented);
    assert!(dropped);
}

#[test]
fn post_present_loss_returns_lost() {
    let scene = Scene::new(4.0, 4.0);
    let clear = Rgba::new(0.0, 0.0, 0.0, 1.0);
    let (outcome, presented, dropped) = run_present(
        |_| {},
        |gpu, renderer, view| renderer.render(gpu, view, &scene, clear),
        lose_device,
    );
    assert!(matches!(outcome, FrameOutcome::Lost));
    assert!(presented);
    assert!(dropped);
}

#[test]
fn healthy_frame_presents_and_returns_presented() {
    let scene = Scene::new(4.0, 4.0);
    let clear = Rgba::new(0.0, 0.0, 0.0, 1.0);
    let (outcome, presented, dropped) = run_present(
        |_| {},
        |gpu, renderer, view| renderer.render(gpu, view, &scene, clear),
        |_| {},
    );
    assert!(matches!(outcome, FrameOutcome::Presented));
    assert!(presented);
    assert!(dropped);
}

#[test]
fn pre_render_loss_skips_render_and_present() {
    let mut rendered = false;
    let (outcome, presented, dropped) = run_present(lose_device, |_, _, _| rendered = true, |_| {});
    assert!(matches!(outcome, FrameOutcome::Lost));
    assert!(!rendered);
    assert!(!presented);
    assert!(dropped);
}
