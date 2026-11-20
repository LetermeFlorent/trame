use super::super::recovery::{FrameResult, Outcome, StepStatus};
use super::FakeSurface;

#[test]
fn lost_surface_recreates_when_device_is_healthy() {
    let mut fake = FakeSurface::new([Outcome::Lost, Outcome::Ready(9)]);
    let (result, logs) = fake.run(false);
    assert!(matches!(result, FrameResult::Ready(9)));
    assert_eq!((fake.reconfigures, fake.recreates), (0, 1));
    assert!(logs[0].contains("lost, recreating"), "{logs:?}");
}

#[test]
fn lost_surface_escalates_when_device_is_lost() {
    let mut fake = FakeSurface::new([Outcome::Lost]);
    fake.device_lost = true;
    let (result, _) = fake.run(false);
    assert!(matches!(result, FrameResult::DeviceLost));
    assert_eq!(fake.recreates, 0);
}

#[test]
fn lost_surface_escalates_when_recreate_fails() {
    let mut fake = FakeSurface::new([Outcome::Lost]);
    fake.recreate_results.push_back(StepStatus::Failed);
    let (result, _) = fake.run(false);
    assert!(matches!(result, FrameResult::DeviceLost));
    assert_eq!(fake.recreates, 1);
}

#[test]
fn lost_surface_escalates_when_second_acquire_fails() {
    let mut fake = FakeSurface::new([Outcome::Lost, Outcome::Validation]);
    let (result, _) = fake.run(false);
    assert!(matches!(result, FrameResult::DeviceLost));
    assert_eq!(fake.recreates, 1);
}

#[test]
fn out_of_memory_surface_recreates_and_recovers() {
    let mut fake = FakeSurface::new([Outcome::OutOfMemory, Outcome::Ready(3)]);
    let (result, logs) = fake.run(false);
    assert!(matches!(result, FrameResult::Ready(3)));
    assert_eq!(fake.recreates, 1);
    assert!(logs[0].contains("out of memory"), "{logs:?}");
}

#[test]
fn persistent_out_of_memory_escalates_to_device_lost() {
    let mut fake = FakeSurface::new([Outcome::OutOfMemory, Outcome::OutOfMemory]);
    let (result, _) = fake.run(false);
    assert!(matches!(result, FrameResult::DeviceLost));
    assert_eq!(fake.recreates, 1);
}

#[test]
fn out_of_memory_during_resize_configure_recreates_surface() {
    let mut fake = FakeSurface::new([Outcome::Ready(11)]);
    fake.reconfigure_results.push_back(StepStatus::OutOfMemory);
    let (result, _) = fake.run(true);
    assert!(matches!(result, FrameResult::Ready(11)));
    assert_eq!((fake.reconfigures, fake.recreates), (1, 1));
}

#[test]
fn resize_out_of_memory_escalates_when_recreate_fails() {
    let mut fake = FakeSurface::new([]);
    fake.reconfigure_results.push_back(StepStatus::OutOfMemory);
    fake.recreate_results.push_back(StepStatus::OutOfMemory);
    let (result, _) = fake.run(true);
    assert!(matches!(result, FrameResult::DeviceLost));
    assert_eq!((fake.reconfigures, fake.recreates), (1, 1));
}

#[test]
fn resize_out_of_memory_escalates_when_second_acquire_fails() {
    let mut fake = FakeSurface::new([Outcome::OutOfMemory]);
    fake.reconfigure_results.push_back(StepStatus::OutOfMemory);
    let (result, _) = fake.run(true);
    assert!(matches!(result, FrameResult::DeviceLost));
    assert_eq!((fake.reconfigures, fake.recreates), (1, 1));
}

#[test]
fn timeout_after_reconfigure_escalates_when_device_lost() {
    let mut fake = FakeSurface::new([Outcome::Outdated, Outcome::Timeout]);
    fake.reconfigure_loses_device = true;
    let (result, _) = fake.run(false);
    assert!(matches!(result, FrameResult::DeviceLost));
}

#[test]
fn resize_failed_configure_escalates_when_device_lost() {
    let mut fake = FakeSurface::new([]);
    fake.reconfigure_loses_device = true;
    fake.reconfigure_results.push_back(StepStatus::Failed);
    let (result, _) = fake.run(true);
    assert!(matches!(result, FrameResult::DeviceLost));
}

#[test]
fn timeout_reconfigure_losing_device_escalates_to_device_lost() {
    let mut fake = FakeSurface::new([Outcome::Timeout]);
    fake.reconfigure_loses_device = true;
    let (result, _) = fake.run(false);
    assert!(matches!(result, FrameResult::DeviceLost));
}
