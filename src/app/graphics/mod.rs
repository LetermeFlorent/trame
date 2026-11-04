#[cfg(feature = "capture")]
mod capture;
mod connect;

use std::error::Error;
use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use connect::{PREFERRED_BACKENDS, connect};
use trame::render::{
    Atlas, Frame, Gpu, ReadbackError, Renderer, Scene, WindowSurface, render_in_format,
};
use trame::theme::Rgba;

#[cfg(feature = "capture")]
use capture::Capture;

pub struct Graphics {
    gpu: Gpu,
    surface: WindowSurface,
    format: wgpu::TextureFormat,
    renderer: Renderer,
    fresh: bool,
    #[cfg(feature = "capture")]
    capture: Option<Capture>,
}

impl Graphics {
    pub fn new(
        event_loop: &ActiveEventLoop,
        window: Arc<Window>,
        atlas: &mut Atlas,
    ) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();
        let (gpu, surface) = connect(event_loop, &window, PREFERRED_BACKENDS)
            .or_else(|_| connect(event_loop, &window, wgpu::Backends::all()))?;
        #[cfg_attr(not(feature = "capture"), expect(unused_mut))]
        let mut surface = WindowSurface::new(&gpu, surface, size.width, size.height)
            .ok_or("the graphics card cannot draw into this window")?;
        #[cfg(feature = "capture")]
        let capture = Capture::from_env();
        #[cfg(feature = "capture")]
        if capture.is_some() {
            surface.allow_copy(&gpu);
        }
        let format = surface.format();
        let renderer = Renderer::new(&gpu, format, atlas);
        Ok(Self {
            gpu,
            surface,
            format,
            renderer,
            fresh: true,
            #[cfg(feature = "capture")]
            capture,
        })
    }

    pub fn take_fresh(&mut self) -> bool {
        std::mem::take(&mut self.fresh)
    }

    pub fn render_image(
        &mut self,
        scene: &Scene,
        atlas: &mut Atlas,
    ) -> Result<Vec<u8>, ReadbackError> {
        self.renderer.sync_atlas(&self.gpu, atlas);
        let clear = Rgba::new(0.0, 0.0, 0.0, 0.0);
        render_in_format(&self.gpu, &mut self.renderer, scene, clear, self.format)
    }

    pub fn is_lost(&self) -> bool {
        self.gpu.is_lost()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.resize(width, height);
    }

    pub fn present(
        &mut self,
        window: &Window,
        scene: &Scene,
        atlas: &mut Atlas,
        clear: Rgba,
    ) -> bool {
        self.renderer.sync_atlas(&self.gpu, atlas);
        let Frame::Ready(texture) = self.surface.acquire(&self.gpu) else {
            return false;
        };
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.renderer.render(&self.gpu, &view, scene, clear);
        #[cfg(feature = "capture")]
        if let Some(capture) = &self.capture {
            capture.save(&self.gpu, &texture.texture);
        }
        window.pre_present_notify();
        self.gpu.queue.present(texture);
        true
    }
}
