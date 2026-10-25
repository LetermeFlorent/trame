use std::error::Error;
use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use trame::render::Gpu;

#[cfg(windows)]
pub(super) const PREFERRED_BACKENDS: wgpu::Backends = wgpu::Backends::DX12;
#[cfg(not(windows))]
pub(super) const PREFERRED_BACKENDS: wgpu::Backends =
    wgpu::Backends::VULKAN.union(wgpu::Backends::METAL);

pub(super) fn connect(
    event_loop: &ActiveEventLoop,
    window: &Arc<Window>,
    backends: wgpu::Backends,
    translucent: bool,
) -> Result<(Gpu, wgpu::Surface<'static>), Box<dyn Error>> {
    let display = Box::new(event_loop.owned_display_handle());
    let mut descriptor = wgpu::InstanceDescriptor::new_with_display_handle(display);
    descriptor.backends = backends.with_env();
    if translucent {
        descriptor.backend_options.dx12.presentation_system =
            wgpu::Dx12SwapchainKind::DxgiFromVisual;
    }
    let instance = wgpu::Instance::new(descriptor);
    let surface = instance.create_surface(window.clone())?;
    let gpu = Gpu::new(instance, Some(&surface))?;
    Ok((gpu, surface))
}
