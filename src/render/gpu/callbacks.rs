use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::log_error;

pub type WakeCallback = Arc<dyn Fn() + Send + Sync>;

pub(crate) fn noop_wake() -> WakeCallback {
    Arc::new(|| {})
}

pub(super) fn attach_callbacks(
    device: &wgpu::Device,
    lost: &Arc<AtomicBool>,
    out_of_memory: &Arc<AtomicBool>,
    closing: &Arc<AtomicBool>,
    wake: WakeCallback,
) {
    let oom_flag = out_of_memory.clone();
    let error_wake = wake.clone();
    device.on_uncaptured_error(Arc::new(move |error| {
        record_device_error(&error, &oom_flag, &*error_wake);
    }));
    let lost_flag = lost.clone();
    let closing_flag = closing.clone();
    device.set_device_lost_callback(move |reason, message| {
        record_device_lost(reason, &message, &closing_flag, &lost_flag, &*wake);
    });
}

pub(super) fn record_device_error(
    error: &wgpu::Error,
    out_of_memory: &AtomicBool,
    wake: &dyn Fn(),
) {
    log_error!("the graphics card reported an error: {error}");
    if matches!(error, wgpu::Error::OutOfMemory { .. }) {
        out_of_memory.store(true, Ordering::Release);
        wake();
    }
}

pub(super) fn record_device_lost(
    reason: wgpu::DeviceLostReason,
    message: &str,
    closing: &AtomicBool,
    lost: &AtomicBool,
    wake: &dyn Fn(),
) {
    if closing.load(Ordering::Acquire) {
        return;
    }
    if message.is_empty() {
        log_error!("the graphics card was lost ({reason:?})");
    } else {
        log_error!("the graphics card was lost ({reason:?}): {message}");
    }
    lost.store(true, Ordering::Release);
    wake();
}
