use std::fmt;

pub struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Debug)]
pub enum GpuError {
    Adapter(wgpu::RequestAdapterError),
    Device(wgpu::RequestDeviceError),
    Surface(wgpu::CreateSurfaceError),
}

impl fmt::Display for GpuError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::Adapter(error) => write!(formatter, "no graphics adapter: {error}"),
            GpuError::Device(error) => write!(formatter, "graphics device refused: {error}"),
            GpuError::Surface(error) => write!(formatter, "window surface refused: {error}"),
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
        let limits = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("trame"),
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }))
        .map_err(GpuError::Device)?;
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub fn headless() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        Self::new(instance, None)
    }
}
