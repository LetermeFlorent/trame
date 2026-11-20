mod configure;
mod offscreen;
mod recovery;
mod resize;

use std::collections::VecDeque;

use super::recovery::{FrameResult, Outcome, StepStatus, SurfaceOps, acquire_step};
use crate::render::OFFSCREEN_FORMAT;

pub(super) struct FakeSurface {
    pub(super) has_surface: bool,
    pub(super) device_lost: bool,
    pub(super) reconfigure_loses_device: bool,
    pub(super) outcomes: VecDeque<Outcome<u32>>,
    pub(super) reconfigure_results: VecDeque<StepStatus>,
    pub(super) recreate_results: VecDeque<StepStatus>,
    pub(super) reconfigures: usize,
    pub(super) recreates: usize,
}

impl FakeSurface {
    pub(super) fn new(outcomes: impl IntoIterator<Item = Outcome<u32>>) -> Self {
        Self {
            has_surface: true,
            device_lost: false,
            reconfigure_loses_device: false,
            outcomes: outcomes.into_iter().collect(),
            reconfigure_results: VecDeque::new(),
            recreate_results: VecDeque::new(),
            reconfigures: 0,
            recreates: 0,
        }
    }

    pub(super) fn run(&mut self, pending: bool) -> (FrameResult<u32>, Vec<String>) {
        let mut logs = Vec::new();
        let (result, _) = acquire_step(self, pending, &mut |line| logs.push(line.to_owned()));
        (result, logs)
    }
}

pub(super) fn test_config(width: u32, height: u32) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: OFFSCREEN_FORMAT,
        width,
        height,
        present_mode: wgpu::PresentMode::AutoVsync,
        desired_maximum_frame_latency: 1,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: Vec::new(),
        color_space: wgpu::SurfaceColorSpace::Srgb,
    }
}

impl SurfaceOps for FakeSurface {
    type Texture = u32;
    fn has_surface(&self) -> bool {
        self.has_surface
    }
    fn acquire_next(&mut self) -> Outcome<u32> {
        self.outcomes.pop_front().unwrap_or(Outcome::Occluded)
    }
    fn reconfigure(&mut self) -> StepStatus {
        self.reconfigures += 1;
        if self.reconfigure_loses_device {
            self.device_lost = true;
        }
        self.reconfigure_results
            .pop_front()
            .unwrap_or(StepStatus::Ok)
    }
    fn recreate(&mut self) -> StepStatus {
        self.recreates += 1;
        let status = self.recreate_results.pop_front().unwrap_or(StepStatus::Ok);
        self.has_surface = status == StepStatus::Ok;
        status
    }
    fn device_lost(&self) -> bool {
        self.device_lost
    }
}

#[test]
fn outdated_surface_reconfigures_and_acquires() {
    let mut fake = FakeSurface::new([Outcome::Outdated, Outcome::Ready(7)]);
    let (result, _) = fake.run(false);
    assert!(matches!(result, FrameResult::Ready(7)));
    assert_eq!((fake.reconfigures, fake.recreates), (1, 0));
}

#[test]
fn suboptimal_surface_reconfigures_once_and_does_not_loop() {
    let mut fake = FakeSurface::new([Outcome::Suboptimal(1), Outcome::Suboptimal(2)]);
    let (result, _) = fake.run(false);
    assert!(matches!(result, FrameResult::Ready(2)));
    assert_eq!((fake.reconfigures, fake.recreates), (1, 0));
}

#[test]
fn timeout_surface_reconfigures_and_skips_with_log() {
    let mut fake = FakeSurface::new([Outcome::Timeout]);
    let (result, logs) = fake.run(false);
    assert!(matches!(result, FrameResult::Skip));
    assert_eq!((fake.reconfigures, fake.recreates), (1, 0));
    assert!(logs[0].contains("timed out"), "{logs:?}");
}

#[test]
fn occluded_surface_skips_without_reconfiguring() {
    let mut fake = FakeSurface::new([Outcome::Occluded]);
    let (result, _) = fake.run(false);
    assert!(matches!(result, FrameResult::Skip));
    assert_eq!((fake.reconfigures, fake.recreates), (0, 0));
}
