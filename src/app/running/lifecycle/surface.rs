use std::error::Error;
use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use trame::preferences::Preferences;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::workspace::WindowSnapshot;

use crate::app::graphics::Graphics;
use crate::app::window;

pub(super) struct Surface {
    pub(super) window: Arc<Window>,
    pub(super) graphics: Graphics,
    pub(super) text: TextSystem,
    pub(super) window_scale: f32,
    pub(super) preferences: Preferences,
}

impl Surface {
    pub(super) fn open(
        event_loop: &ActiveEventLoop,
        saved: Option<WindowSnapshot>,
        preferences: Preferences,
    ) -> Result<Self, Box<dyn Error>> {
        let window = Arc::new(event_loop.create_window(window::restored_attributes(saved))?);
        window::round_corners(&window);
        let window_scale = window.scale_factor() as f32;
        let font = preferences.font_pixels(window_scale);
        let mut text = TextSystem::new(font, DEFAULT_LINE_HEIGHT)?;
        let graphics = Graphics::new(event_loop, window.clone(), text.atlas_mut())?;
        Ok(Self {
            window,
            graphics,
            text,
            window_scale,
            preferences,
        })
    }
}
