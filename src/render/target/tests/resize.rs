use super::super::recovery::{FrameResult, Outcome, StepStatus, acquire_step};
use super::super::surface::{restore_pending_size, take_pending_size};
use super::{FakeSurface, test_config};

fn step_with_pending(
    config: &mut wgpu::SurfaceConfiguration,
    pending: &mut Option<(u32, u32)>,
    fake: &mut FakeSurface,
) -> FrameResult<u32> {
    let saved = take_pending_size(config, pending);
    let mut logs = Vec::new();
    let (result, restore) = acquire_step(fake, saved.is_some(), &mut |line| {
        logs.push(line.to_owned())
    });
    if restore {
        restore_pending_size(config, pending, saved);
    }
    result
}

#[test]
fn failed_pending_reconfigure_skip_preserves_size_and_restores_pending() {
    let mut config = test_config(800, 600);
    let mut pending = Some((1200, 900));
    let mut fake = FakeSurface::new([Outcome::Ready(1)]);
    fake.reconfigure_results.push_back(StepStatus::Failed);

    let result = step_with_pending(&mut config, &mut pending, &mut fake);
    assert!(matches!(result, FrameResult::Skip));
    assert_eq!((config.width, config.height), (800, 600));
    assert_eq!(pending, Some((1200, 900)));
}

#[test]
fn occluded_acquire_after_successful_pending_reconfigure_keeps_new_size() {
    let mut config = test_config(800, 600);
    let mut pending = Some((1200, 900));
    let mut fake = FakeSurface::new([Outcome::Occluded]);
    fake.reconfigure_results.push_back(StepStatus::Ok);

    let result = step_with_pending(&mut config, &mut pending, &mut fake);
    assert!(matches!(result, FrameResult::Skip));
    assert_eq!((config.width, config.height), (1200, 900));
    assert_eq!(pending, None);
}

#[test]
fn failed_pending_reconfigure_on_lost_device_returns_device_lost() {
    let mut config = test_config(800, 600);
    let mut pending = Some((1200, 900));
    let mut fake = FakeSurface::new([]);
    fake.reconfigure_loses_device = true;
    fake.reconfigure_results.push_back(StepStatus::Failed);

    let result = step_with_pending(&mut config, &mut pending, &mut fake);
    assert!(matches!(result, FrameResult::DeviceLost));
}
