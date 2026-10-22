use std::error::Error;
use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use trame::render::{
    Atlas, Frame, Gpu, ReadbackError, Renderer, Scene, WindowSurface, render_in_format,
};
use trame::theme::Rgba;

#[cfg(feature = "capture")]
use super::capture::Capture;

pub struct Graphics {
    gpu: Gpu,
    surface: WindowSurface,
    renderer: Renderer,
    #[cfg(feature = "capture")]
    capture: Option<Capture>,
}

impl Graphics {
    pub fn new(
        event_loop: &ActiveEventLoop,
        window: Arc<Window>,
        atlas: &Atlas,
    ) -> Result<Self, Box<dyn Error>> {
        let display = Box::new(event_loop.owned_display_handle());
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(display));
        let size = window.inner_size();
        let surface = instance.create_surface(window)?;
        let gpu = Gpu::new(instance, Some(&surface))?;
        #[cfg_attr(not(feature = "capture"), expect(unused_mut))]
        let mut surface = WindowSurface::new(&gpu, surface, size.width, size.height)
            .ok_or("the graphics card cannot draw into this window")?;
        #[cfg(feature = "capture")]
        let capture = Capture::from_env();
        #[cfg(feature = "capture")]
        if capture.is_some() {
            surface.allow_copy(&gpu);
        }
        let renderer = Renderer::new(&gpu, surface.format(), atlas);
        Ok(Self {
            gpu,
            surface,
            renderer,
            #[cfg(feature = "capture")]
            capture,
        })
    }

    pub fn render_image(
        &mut self,
        scene: &Scene,
        atlas: &mut Atlas,
    ) -> Result<Vec<u8>, ReadbackError> {
        self.renderer.sync_atlas(&self.gpu, atlas);
        let clear = Rgba::new(0.0, 0.0, 0.0, 0.0);
        let format = self.surface.format();
        render_in_format(&self.gpu, &mut self.renderer, scene, clear, format)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.resize(&self.gpu, width, height);
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
