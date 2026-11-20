mod capture;
mod connect;
mod plan;
mod recovery;
#[cfg(test)]
#[path = "tests/mod.rs"]
mod render_tests;
mod session;

use std::error::Error;
use std::sync::Arc;

use winit::event_loop::{ActiveEventLoop, OwnedDisplayHandle};
use winit::window::Window;

use capture::Capture;
use connect::open_session;
use recovery::GraphicsState;
use session::RenderFrame;
use trame::log_error;
use trame::render::{Atlas, ReadbackError, Scene, WindowSurface};
use trame::theme::Rgba;

pub struct Graphics {
    window: Arc<Window>,
    display: OwnedDisplayHandle,
    state: GraphicsState<WindowSurface>,
    capture: Option<Capture>,
}

impl Graphics {
    pub fn new(
        event_loop: &ActiveEventLoop,
        window: Arc<Window>,
        atlas: &mut Atlas,
    ) -> Result<Self, Box<dyn Error>> {
        let display = event_loop.owned_display_handle();
        let capture = Capture::from_env();
        let session = open_session(&display, &window, atlas, capture.is_some(), false)?;
        Ok(Self {
            window,
            display,
            state: GraphicsState::new(session),
            capture,
        })
    }

    pub fn take_fresh(&mut self) -> bool {
        std::mem::take(&mut self.state.fresh)
    }

    pub fn render_image(
        &mut self,
        scene: &Scene,
        atlas: &mut Atlas,
    ) -> Result<Vec<u8>, ReadbackError> {
        let allow_copy = self.capture.is_some();
        let display = &self.display;
        let window = &self.window;
        let result = self.state.render_image_with(
            atlas,
            |active, atlas| active.render_offscreen(scene, atlas),
            |atlas, prefer_software| {
                open_session(display, window, atlas, allow_copy, prefer_software)
            },
            &mut |line| log_error!("{line}"),
        );
        if std::mem::take(&mut self.state.redraw) {
            self.window.request_redraw();
        }
        result
    }

    pub fn is_lost(&self) -> bool {
        self.state.failed
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(active) = &mut self.state.active {
            active.surface.resize(width, height);
        }
    }

    pub fn max_texture_size(&self) -> u32 {
        self.state
            .active
            .as_ref()
            .map_or(u32::MAX, |s| s.gpu.device.limits().max_texture_dimension_2d)
    }

    pub fn set_picture(&mut self, image: Option<(u32, u32, &[u8])>) {
        if let Some(active) = &mut self.state.active {
            active.renderer.set_picture(&active.gpu, image);
        }
    }

    pub fn present(
        &mut self,
        _window: &Window,
        scene: &Scene,
        atlas: &mut Atlas,
        clear: Rgba,
    ) -> bool {
        let allow_copy = self.capture.is_some();
        let display = &self.display;
        let window = &self.window;
        let capture = self.capture.as_ref();
        let presented = self.state.present_with(
            atlas,
            |active, atlas| {
                active.acquire_and_present(
                    window,
                    RenderFrame {
                        scene,
                        atlas,
                        clear,
                    },
                    capture,
                )
            },
            |atlas, prefer_software| {
                open_session(display, window, atlas, allow_copy, prefer_software)
            },
            &mut |line| log_error!("{line}"),
        );
        if std::mem::take(&mut self.state.redraw) {
            self.window.request_redraw();
        }
        presented
    }
}
