#[cfg(test)]
#[path = "connect_tests.rs"]
mod render_tests;

use std::env;
use std::error::Error;
use std::sync::Arc;

use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use super::plan::{Attempt, planned_attempts, try_connect};
use super::session::Session;
use trame::log_error;
use trame::render::{Atlas, Gpu, Renderer, WakeCallback, WindowSurface};

#[cfg(windows)]
const PREFERRED_BACKENDS: wgpu::Backends = wgpu::Backends::VULKAN;
#[cfg(target_os = "macos")]
const PREFERRED_BACKENDS: wgpu::Backends = wgpu::Backends::METAL;
#[cfg(not(any(windows, target_os = "macos")))]
const PREFERRED_BACKENDS: wgpu::Backends = wgpu::Backends::VULKAN;

const FALLBACK_ADAPTER_ENV: &str = "TRAME_FALLBACK_ADAPTER";

pub(super) struct Connected {
    pub(super) gpu: Gpu,
    pub(super) surface: WindowSurface,
    pub(super) software: bool,
}

pub(super) fn connect(
    display: &OwnedDisplayHandle,
    window: &Arc<Window>,
    size: (u32, u32),
    wake: WakeCallback,
    prefer_software: bool,
) -> Result<Connected, Box<dyn Error>> {
    let env_backends = wgpu::Backends::from_env();
    let force_software = prefer_software
        || env::var_os(FALLBACK_ADAPTER_ENV).is_some_and(|value| !value.is_empty() && value != "0");
    let enabled = wgpu::Instance::enabled_backend_features();
    let attempts = planned_attempts(PREFERRED_BACKENDS, enabled, env_backends, force_software);
    try_connect(
        &attempts,
        |attempt| open_attempt(display, window, size, attempt, wake.clone()),
        &mut |line| log_error!("{line}"),
    )
}

fn open_attempt(
    display: &OwnedDisplayHandle,
    window: &Arc<Window>,
    size: (u32, u32),
    attempt: Attempt,
    wake: WakeCallback,
) -> Result<(Connected, String), Box<dyn Error>> {
    let mut descriptor =
        wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display.clone()));
    descriptor.backends = attempt.backends;
    let instance = wgpu::Instance::new(descriptor);
    let surface = instance.create_surface(window.clone())?;
    let gpu = Gpu::new_with(instance, Some(&surface), attempt.software, wake)?;
    let name = gpu.adapter_info().name;
    let surface = WindowSurface::new(&gpu, surface, size.0, size.1)
        .ok_or("the graphics card cannot draw into this window")?;
    Ok((
        Connected {
            gpu,
            surface,
            software: attempt.software,
        },
        name,
    ))
}

pub(super) fn open_session(
    display: &OwnedDisplayHandle,
    window: &Arc<Window>,
    atlas: &mut Atlas,
    allow_copy: bool,
    prefer_software: bool,
) -> Result<Session<WindowSurface>, Box<dyn Error>> {
    let size = window.inner_size();
    let wake = window_waker(window);
    let dims = (size.width, size.height);
    let mut link = connect(display, window, dims, wake, prefer_software)?;
    if allow_copy && !link.surface.allow_copy(&link.gpu) {
        return Err("the graphics card cannot copy from this window".into());
    }
    let format = link.surface.format();
    let renderer = Renderer::new(&link.gpu, format, atlas);
    Ok(Session {
        gpu: link.gpu,
        surface: link.surface,
        format,
        renderer,
        software: link.software,
    })
}

fn window_waker(window: &Arc<Window>) -> WakeCallback {
    let weak = Arc::downgrade(window);
    Arc::new(move || {
        if let Some(window) = weak.upgrade() {
            window.request_redraw();
        }
    })
}
