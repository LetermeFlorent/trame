use super::recovery::{Outcome, StepStatus, SurfaceOps};
use super::surface::WindowSurface;
use crate::log_error;
use crate::render::gpu::Gpu;

pub(super) struct WgpuSurface<'a, F> {
    pub(super) owner: &'a mut WindowSurface,
    pub(super) gpu: &'a Gpu,
    pub(super) recreate: &'a mut F,
}

impl<F> SurfaceOps for WgpuSurface<'_, F>
where
    F: FnMut(&wgpu::Instance) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError>,
{
    type Texture = wgpu::SurfaceTexture;

    fn has_surface(&self) -> bool {
        self.owner.surface.is_some()
    }

    fn acquire_next(&mut self) -> Outcome<wgpu::SurfaceTexture> {
        let Some(surface) = &self.owner.surface else {
            return Outcome::Lost;
        };
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => Outcome::Ready(texture),
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Outcome::Suboptimal(texture),
            wgpu::CurrentSurfaceTexture::Outdated => Outcome::Outdated,
            wgpu::CurrentSurfaceTexture::Lost => Outcome::Lost,
            wgpu::CurrentSurfaceTexture::Timeout => Outcome::Timeout,
            wgpu::CurrentSurfaceTexture::Occluded => Outcome::Occluded,
            wgpu::CurrentSurfaceTexture::Validation => validation_outcome(self.gpu),
        }
    }

    fn reconfigure(&mut self) -> StepStatus {
        let Some(surface) = &self.owner.surface else {
            return StepStatus::Failed;
        };
        configure_surface(self.gpu, surface, &self.owner.config)
    }

    fn recreate(&mut self) -> StepStatus {
        self.owner.surface = None;
        let Ok(surface) = (self.recreate)(&self.gpu.instance) else {
            return StepStatus::Failed;
        };
        let status = configure_surface(self.gpu, &surface, &self.owner.config);
        if status == StepStatus::Ok {
            self.owner.surface = Some(surface);
        }
        status
    }

    fn device_lost(&self) -> bool {
        self.gpu.is_lost()
    }
}

pub(super) fn validation_outcome<T>(gpu: &Gpu) -> Outcome<T> {
    if gpu.is_lost() {
        Outcome::Lost
    } else if gpu.take_out_of_memory() {
        Outcome::OutOfMemory
    } else {
        Outcome::Validation
    }
}

pub(super) fn configure_surface(
    gpu: &Gpu,
    surface: &wgpu::Surface<'_>,
    config: &wgpu::SurfaceConfiguration,
) -> StepStatus {
    configure_with(
        gpu,
        || surface.configure(&gpu.device, config),
        &mut |line| log_error!("{line}"),
    )
}

pub(super) fn configure_with(
    gpu: &Gpu,
    configure: impl FnOnce(),
    log: &mut dyn FnMut(&str),
) -> StepStatus {
    let scope = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
    configure();
    if let Some(error) = pollster::block_on(scope.pop()) {
        log(&format!("the window surface configuration failed: {error}"));
        return StepStatus::Failed;
    }
    configure_status(gpu)
}

pub(super) fn configure_status(gpu: &Gpu) -> StepStatus {
    if gpu.is_lost() {
        StepStatus::Failed
    } else if gpu.take_out_of_memory() {
        StepStatus::OutOfMemory
    } else {
        StepStatus::Ok
    }
}
