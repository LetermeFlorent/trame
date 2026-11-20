use super::recovery::{FrameResult, StepStatus, acquire_step};
use super::wgpu_surface::{WgpuSurface, configure_surface};
use crate::log_error;
use crate::render::gpu::Gpu;

pub struct WindowSurface {
    pub(super) surface: Option<wgpu::Surface<'static>>,
    pub(super) config: wgpu::SurfaceConfiguration,
    pending: Option<(u32, u32)>,
}

pub enum Frame {
    Ready(wgpu::SurfaceTexture),
    Skip,
    DeviceLost,
}

impl WindowSurface {
    pub fn new(
        gpu: &Gpu,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Option<Self> {
        let capabilities = surface.get_capabilities(&gpu.adapter);
        let mut config = surface.get_default_config(&gpu.adapter, width.max(1), height.max(1))?;
        if let Some(format) = preferred_format(&capabilities.formats) {
            config.format = format;
        }
        config.present_mode = wgpu::PresentMode::AutoVsync;
        if capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Opaque)
        {
            config.alpha_mode = wgpu::CompositeAlphaMode::Opaque;
        }
        config.desired_maximum_frame_latency = 1;
        if configure_surface(gpu, &surface, &config) != StepStatus::Ok {
            return None;
        }
        Some(Self {
            surface: Some(surface),
            config,
            pending: None,
        })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.pending = ((width, height) != self.size()).then_some((width, height));
    }

    pub fn allow_copy(&mut self, gpu: &Gpu) -> bool {
        let Some(surface) = &self.surface else {
            return false;
        };
        let usages = surface.get_capabilities(&gpu.adapter).usages;
        allow_copy_with(gpu, &mut self.config, usages, |gpu, config| {
            configure_surface(gpu, surface, config)
        })
    }

    pub fn acquire<F>(&mut self, gpu: &Gpu, mut recreate: F) -> Frame
    where
        F: FnMut(&wgpu::Instance) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError>,
    {
        let saved = take_pending_size(&mut self.config, &mut self.pending);
        let mut driver = WgpuSurface {
            owner: self,
            gpu,
            recreate: &mut recreate,
        };
        let (result, restore) = acquire_step(&mut driver, saved.is_some(), &mut |line| {
            log_error!("{line}")
        });
        if restore {
            restore_pending_size(&mut self.config, &mut self.pending, saved);
        }
        match result {
            FrameResult::Ready(texture) => Frame::Ready(texture),
            FrameResult::Skip => Frame::Skip,
            FrameResult::DeviceLost => Frame::DeviceLost,
        }
    }
}

pub(super) fn allow_copy_with(
    gpu: &Gpu,
    config: &mut wgpu::SurfaceConfiguration,
    usages: wgpu::TextureUsages,
    configure: impl FnOnce(&Gpu, &wgpu::SurfaceConfiguration) -> StepStatus,
) -> bool {
    if !usages.contains(wgpu::TextureUsages::COPY_SRC) {
        return false;
    }
    let previous = config.usage;
    config.usage |= wgpu::TextureUsages::COPY_SRC;
    if configure(gpu, config) == StepStatus::Ok {
        return true;
    }
    config.usage = previous;
    false
}

pub(super) fn take_pending_size(
    config: &mut wgpu::SurfaceConfiguration,
    pending: &mut Option<(u32, u32)>,
) -> Option<((u32, u32), (u32, u32))> {
    let next = pending.take()?;
    let previous = (config.width, config.height);
    config.width = next.0;
    config.height = next.1;
    Some((previous, next))
}

pub(super) fn restore_pending_size(
    config: &mut wgpu::SurfaceConfiguration,
    pending: &mut Option<(u32, u32)>,
    saved: Option<((u32, u32), (u32, u32))>,
) {
    if let Some((previous, next)) = saved {
        config.width = previous.0;
        config.height = previous.1;
        *pending = Some(next);
    }
}

fn preferred_format(formats: &[wgpu::TextureFormat]) -> Option<wgpu::TextureFormat> {
    let wanted = [
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba8Unorm,
    ];
    wanted
        .into_iter()
        .find(|format| formats.contains(format))
        .or_else(|| formats.first().copied())
}
