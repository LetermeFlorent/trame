use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::super::recovery::{Outcome, StepStatus};
use super::super::surface::allow_copy_with;
use super::super::wgpu_surface::{configure_status, configure_with, validation_outcome};
use super::test_config;
use crate::render::{Gpu, OFFSCREEN_FORMAT};

#[test]
fn configure_status_checks_is_lost_before_out_of_memory() {
    let gpu = Gpu::headless().expect("headless gpu");
    gpu.mark_out_of_memory();
    gpu.device.destroy();
    assert!(gpu.poll_lost());
    let status = configure_status(&gpu);
    assert!(matches!(status, StepStatus::Failed));
    assert!(gpu.take_out_of_memory());
}

#[test]
fn validation_outcome_checks_is_lost_before_out_of_memory() {
    let gpu = Gpu::headless().expect("headless gpu");
    assert!(matches!(
        validation_outcome::<u32>(&gpu),
        Outcome::Validation
    ));
    gpu.mark_out_of_memory();
    assert!(matches!(
        validation_outcome::<u32>(&gpu),
        Outcome::OutOfMemory
    ));
    assert!(!gpu.take_out_of_memory());

    gpu.mark_out_of_memory();
    gpu.device.destroy();
    assert!(gpu.poll_lost());
    assert!(matches!(validation_outcome::<u32>(&gpu), Outcome::Lost));
    assert!(gpu.take_out_of_memory());
}

#[test]
fn configure_with_captures_validation_error_and_logs_it() {
    let gpu = Gpu::headless().expect("headless gpu");
    let uncaptured = Arc::new(AtomicBool::new(false));
    let seen = uncaptured.clone();
    gpu.device.on_uncaptured_error(Arc::new(move |_| {
        seen.store(true, Ordering::Release);
    }));
    let mut logs = Vec::new();
    let status = configure_with(
        &gpu,
        || {
            let _ = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("invalid zero extent"),
                size: wgpu::Extent3d {
                    width: 0,
                    height: 0,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        },
        &mut |line| logs.push(line.to_owned()),
    );
    assert!(matches!(status, StepStatus::Failed));
    assert!(!uncaptured.load(Ordering::Acquire));
    assert_eq!(logs.len(), 1);
    assert!(logs[0].contains("configuration failed"), "{logs:?}");
}

#[test]
fn configure_with_returns_ok_when_healthy() {
    let gpu = Gpu::headless().expect("headless gpu");
    let mut logs = Vec::new();
    let status = configure_with(&gpu, || {}, &mut |line| logs.push(line.to_owned()));
    assert!(matches!(status, StepStatus::Ok));
    assert!(logs.is_empty(), "{logs:?}");
}

#[test]
fn allow_copy_without_copy_src_returns_false_without_consuming_oom() {
    let gpu = Gpu::headless().expect("headless gpu");
    gpu.mark_out_of_memory();
    let mut config = test_config(800, 600);
    let ok = allow_copy_with(
        &gpu,
        &mut config,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
        |_, _| panic!("configure must not run without COPY_SRC"),
    );
    assert!(!ok);
    assert!(gpu.take_out_of_memory());
    assert!(!config.usage.contains(wgpu::TextureUsages::COPY_SRC));
}

#[test]
fn allow_copy_with_copy_src_enables_usage_and_configures() {
    let gpu = Gpu::headless().expect("headless gpu");
    let mut config = test_config(800, 600);
    let usages = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC;
    let ok = allow_copy_with(&gpu, &mut config, usages, |_, cfg| {
        assert!(cfg.usage.contains(wgpu::TextureUsages::COPY_SRC));
        StepStatus::Ok
    });
    assert!(ok);
}

#[test]
fn allow_copy_restores_previous_usage_unless_configure_returns_ok() {
    let gpu = Gpu::headless().expect("headless gpu");
    let usages = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC;
    for status in [StepStatus::Failed, StepStatus::OutOfMemory] {
        let mut config = test_config(800, 600);
        let previous = config.usage;
        let ok = allow_copy_with(&gpu, &mut config, usages, |_, cfg| {
            assert!(cfg.usage.contains(wgpu::TextureUsages::COPY_SRC));
            status
        });
        assert!(!ok);
        assert_eq!(config.usage, previous);
    }
}
