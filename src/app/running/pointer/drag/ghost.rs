use std::error::Error;

use winit::event_loop::ActiveEventLoop;
use winit::window::{Cursor, CursorIcon, CustomCursor};

use trame::log_error;
use trame::ui::drop::{ghost_scene, stamp_arrow, straight_alpha};

use super::Drag;
use crate::app::running::Running;

#[derive(Default)]
pub(in crate::app::running) enum Ghost {
    #[default]
    Idle,
    Cursor(CustomCursor),
    Painted,
}

impl Ghost {
    pub(in crate::app::running) fn painted(&self) -> bool {
        !matches!(self, Ghost::Cursor(_))
    }

    pub(super) fn cursor(&self) -> Cursor {
        match self {
            Ghost::Cursor(cursor) => Cursor::Custom(cursor.clone()),
            _ => Cursor::Icon(CursorIcon::Default),
        }
    }
}

impl Running {
    pub(in crate::app::running) fn lift_ghost(&mut self, event_loop: &ActiveEventLoop) {
        let (Ghost::Idle, Some(Drag::Tab(tab, _))) = (&self.pointer.ghost, &self.drag) else {
            return;
        };
        let label = self.tab_title(*tab);
        self.pointer.ghost = match self.ghost_cursor(event_loop, &label) {
            Ok(cursor) => Ghost::Cursor(cursor),
            Err(error) => {
                log_error!("the dragged tab is drawn in the window instead: {error}");
                Ghost::Painted
            }
        };
        self.refresh_hover();
        self.window.request_redraw();
    }

    fn ghost_cursor(
        &mut self,
        event_loop: &ActiveEventLoop,
        label: &str,
    ) -> Result<CustomCursor, Box<dyn Error>> {
        let scene = ghost_scene(&mut self.text, &self.theme, self.metrics, label);
        let mut pixels = self.graphics.render_image(&scene, self.text.atlas_mut())?;
        let width = scene.width() as u16;
        let height = scene.height() as u16;
        straight_alpha(&mut pixels);
        stamp_arrow(&mut pixels, u32::from(width), self.metrics);
        let source = CustomCursor::from_rgba(pixels, width, height, 0, 0)?;
        Ok(event_loop.create_custom_cursor(source))
    }
}
