mod callbacks;
mod error;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::log_error;
use callbacks::{attach_callbacks, noop_wake};

pub use callbacks::WakeCallback;
pub use error::GpuError;

const DESCRIPTOR_SLOTS: u32 = 4096;

pub struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    lost: Arc<AtomicBool>,
    out_of_memory: Arc<AtomicBool>,
    closing: Arc<AtomicBool>,
}

impl Drop for Gpu {
    fn drop(&mut self) {
        self.closing.store(true, Ordering::Release);
    }
}

impl Gpu {
    pub fn new(
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, GpuError> {
        Self::new_with(instance, surface, false, noop_wake())
    }

    pub fn new_with(
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
        force_fallback_adapter: bool,
        wake: WakeCallback,
    ) -> Result<Self, GpuError> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter,
            compatible_surface: surface,
            ..Default::default()
        }))
        .map_err(GpuError::Adapter)?;
        let available = adapter.limits();
        let limits = wgpu::Limits {
            max_non_sampler_bindings: DESCRIPTOR_SLOTS.min(available.max_non_sampler_bindings),
            ..wgpu::Limits::downlevel_webgl2_defaults().using_resolution(available)
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("trame"),
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }))
        .map_err(GpuError::Device)?;
        let lost = Arc::new(AtomicBool::new(false));
        let out_of_memory = Arc::new(AtomicBool::new(false));
        let closing = Arc::new(AtomicBool::new(false));
        attach_callbacks(&device, &lost, &out_of_memory, &closing, wake);
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            lost,
            out_of_memory,
            closing,
        })
    }

    pub fn is_lost(&self) -> bool {
        self.lost.load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(crate) fn poll_lost(&self) -> bool {
        if !self.is_lost() {
            let _ = self.device.poll(wgpu::PollType::Poll);
        }
        self.is_lost()
    }

    #[cfg(test)]
    pub(crate) fn mark_out_of_memory(&self) {
        self.out_of_memory.store(true, Ordering::Release);
    }

    pub fn take_out_of_memory(&self) -> bool {
        self.out_of_memory.swap(false, Ordering::AcqRel)
    }

    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub fn headless() -> Result<Self, GpuError> {
        Self::headless_with(wgpu::Instance::enabled_backend_features(), &mut |line| {
            log_error!("{line}");
        })
    }

    pub fn headless_with(
        backends: wgpu::Backends,
        log: &mut dyn FnMut(&str),
    ) -> Result<Self, GpuError> {
        let instance = headless_instance(backends);
        match Self::new_with(instance, None, false, noop_wake()) {
            Ok(gpu) => Ok(gpu),
            Err(error) => {
                log(&format!(
                    "no hardware graphics adapter worked ({error}), falling back to software adapter"
                ));
                let gpu = Self::headless_software()?;
                log(&format!(
                    "using the software graphics adapter: {}",
                    gpu.adapter_info().name
                ));
                Ok(gpu)
            }
        }
    }

    pub fn headless_software() -> Result<Self, GpuError> {
        let instance = headless_instance(wgpu::Instance::enabled_backend_features());
        Self::new_with(instance, None, true, noop_wake())
    }
}

fn headless_instance(backends: wgpu::Backends) -> wgpu::Instance {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = backends;
    wgpu::Instance::new(descriptor)
}
