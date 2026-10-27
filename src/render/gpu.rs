use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const DESCRIPTOR_SLOTS: u32 = 4096;

pub struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    lost: Arc<AtomicBool>,
}

#[derive(Debug)]
pub enum GpuError {
    Adapter(wgpu::RequestAdapterError),
    Device(wgpu::RequestDeviceError),
}

impl fmt::Display for GpuError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::Adapter(error) => write!(formatter, "no graphics adapter: {error}"),
            GpuError::Device(error) => write!(formatter, "graphics device refused: {error}"),
        }
    }
}

impl std::error::Error for GpuError {}

impl Gpu {
    pub fn new(
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, GpuError> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: surface,
            ..Default::default()
        }))
        .map_err(GpuError::Adapter)?;
        let limits = wgpu::Limits {
            max_non_sampler_bindings: DESCRIPTOR_SLOTS,
            ..wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("trame"),
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }))
        .map_err(GpuError::Device)?;
        device.on_uncaptured_error(Arc::new(|error| {
            eprintln!("trame: the graphics card reported an error: {error}");
        }));
        let lost = Arc::new(AtomicBool::new(false));
        let flag = lost.clone();
        device.set_device_lost_callback(move |reason, message| {
            if reason == wgpu::DeviceLostReason::Unknown {
                eprintln!("trame: the graphics card was lost: {message}");
                flag.store(true, Ordering::Release);
            }
        });
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            lost,
        })
    }

    pub fn is_lost(&self) -> bool {
        self.lost.load(Ordering::Acquire)
    }

    pub fn headless() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        Self::new(instance, None)
    }
}
