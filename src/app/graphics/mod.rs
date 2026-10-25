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
    surface: Option<WindowSurface>,
    format: wgpu::TextureFormat,
    renderer: Renderer,
    built_translucent: bool,
    wanted_translucent: bool,
    fresh: bool,
    #[cfg(feature = "capture")]
    capture: Option<Capture>,
}

impl Graphics {
    pub fn new(
        event_loop: &ActiveEventLoop,
        window: Arc<Window>,
        atlas: &Atlas,
        translucent: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();
        let (gpu, surface) = connect(event_loop, &window, PREFERRED_BACKENDS, translucent)
            .or_else(|_| connect(event_loop, &window, wgpu::Backends::all(), translucent))?;
        #[cfg_attr(not(feature = "capture"), expect(unused_mut))]
        let mut surface = WindowSurface::new(&gpu, surface, size.width, size.height, translucent)
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
            surface: Some(surface),
            format,
            renderer,
            built_translucent: translucent,
            wanted_translucent: translucent,
            fresh: true,
            #[cfg(feature = "capture")]
            capture,
        })
    }

    pub fn reopen(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: Arc<Window>,
        atlas: &mut Atlas,
        translucent: bool,
    ) -> Result<(), Box<dyn Error>> {
        self.surface = None;
        self.built_translucent = self.wanted_translucent;
        let fresh = Self::new(event_loop, window.clone(), atlas, translucent)
            .or_else(|_| Self::new(event_loop, window, atlas, false))?;
        *self = fresh;
        self.wanted_translucent = self.built_translucent;
        Ok(())
    }

    pub fn want_translucent(&mut self, translucent: bool) {
        self.wanted_translucent = translucent;
    }

    pub fn needs_reopen(&self) -> bool {
        self.wanted_translucent != self.built_translucent
    }

    pub fn take_fresh(&mut self) -> bool {
        std::mem::take(&mut self.fresh)
    }

    pub fn is_translucent(&self) -> bool {
        self.surface
            .as_ref()
            .is_some_and(WindowSurface::is_translucent)
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
        self.gpu.is_lost() || self.surface.is_none()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(surface) = &mut self.surface {
            surface.resize(&self.gpu, width, height);
        }
    }

    pub fn present(
        &mut self,
        window: &Window,
        scene: &Scene,
        atlas: &mut Atlas,
        clear: Rgba,
    ) -> bool {
        self.renderer.sync_atlas(&self.gpu, atlas);
        let Some(surface) = &mut self.surface else {
            return false;
        };
        let Frame::Ready(texture) = surface.acquire(&self.gpu) else {
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
