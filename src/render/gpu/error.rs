use std::fmt;

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
