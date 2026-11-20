use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use super::super::callbacks::{record_device_error, record_device_lost};
use super::super::{Gpu, headless_instance};
use super::render_white_glyph;
use crate::render::{Atlas, OFFSCREEN_FORMAT, Renderer};

#[test]
fn destroyed_device_marks_gpu_lost_after_poll_and_wakes() {
    let woken = Arc::new(AtomicUsize::new(0));
    let wake_count = woken.clone();
    let instance = headless_instance(wgpu::Instance::enabled_backend_features());
    let gpu = Gpu::new_with(
        instance,
        None,
        false,
        Arc::new(move || {
            wake_count.fetch_add(1, Ordering::Relaxed);
        }),
    )
    .expect("adapter");
    assert!(!gpu.is_lost());
    gpu.device.destroy();
    assert!(gpu.poll_lost());
    assert_eq!(woken.load(Ordering::Relaxed), 1);
}

#[test]
fn normal_drop_ignores_destroyed_callback() {
    let closing = AtomicBool::new(true);
    let lost = AtomicBool::new(false);
    let woken = AtomicUsize::new(0);
    record_device_lost(
        wgpu::DeviceLostReason::Destroyed,
        "dropped",
        &closing,
        &lost,
        &|| {
            woken.fetch_add(1, Ordering::Relaxed);
        },
    );
    assert!(!lost.load(Ordering::Relaxed));
    assert_eq!(woken.load(Ordering::Relaxed), 0);
}

#[test]
fn destroyed_callback_marks_lost_and_wakes_while_not_dropped() {
    let closing = AtomicBool::new(false);
    let lost = AtomicBool::new(false);
    let woken = AtomicUsize::new(0);
    record_device_lost(
        wgpu::DeviceLostReason::Destroyed,
        "driver reset",
        &closing,
        &lost,
        &|| {
            woken.fetch_add(1, Ordering::Relaxed);
        },
    );
    assert!(lost.load(Ordering::Relaxed));
    assert_eq!(woken.load(Ordering::Relaxed), 1);
}

#[test]
fn normal_gpu_drop_does_not_wake() {
    let woken = Arc::new(AtomicUsize::new(0));
    let wake_count = woken.clone();
    let instance = headless_instance(wgpu::Instance::enabled_backend_features());
    let gpu = Gpu::new_with(
        instance,
        None,
        false,
        Arc::new(move || {
            wake_count.fetch_add(1, Ordering::Relaxed);
        }),
    )
    .expect("adapter");
    drop(gpu);
    assert_eq!(woken.load(Ordering::Relaxed), 0);
}

#[test]
fn out_of_memory_error_marks_flag_and_wakes() {
    let oom = AtomicBool::new(false);
    let woken = AtomicUsize::new(0);
    let error = wgpu::Error::OutOfMemory {
        source: Box::new(std::io::Error::other("simulated oom")),
    };
    record_device_error(&error, &oom, &|| {
        woken.fetch_add(1, Ordering::Relaxed);
    });
    assert!(oom.swap(false, Ordering::AcqRel));
    assert!(!oom.load(Ordering::Relaxed));
    assert_eq!(woken.load(Ordering::Relaxed), 1);
}

#[test]
fn lost_device_recreates_gpu_pipelines_and_glyph_atlas() {
    let gpu = Gpu::headless().expect("initial gpu");
    let mut atlas = Atlas::new();
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, &mut atlas);
    let before = render_white_glyph(&gpu, &mut renderer, &mut atlas);
    assert!(before[0] > 0);
    gpu.device.destroy();
    assert!(gpu.poll_lost());
    drop(renderer);
    drop(gpu);

    let recovered = Gpu::headless().expect("recovered gpu");
    assert!(!recovered.is_lost());
    let mut renderer = Renderer::new(&recovered, OFFSCREEN_FORMAT, &mut atlas);
    atlas.invalidate();
    assert!(atlas.finish_frame());
    atlas.clear();
    let after = render_white_glyph(&recovered, &mut renderer, &mut atlas);
    assert_eq!(after, before);
}
