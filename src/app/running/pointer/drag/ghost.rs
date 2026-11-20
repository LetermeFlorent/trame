#[cfg(test)]
#[path = "ghost_tests.rs"]
mod tests;

use std::error::Error;

use winit::event_loop::ActiveEventLoop;
use winit::window::{Cursor, CursorIcon, CustomCursor};

use trame::log_error;
use trame::render::{Atlas, ReadbackError, Scene};
use trame::text::TextSystem;
use trame::theme::Theme;
use trame::ui::UiMetrics;
use trame::ui::drop::{ghost_scene, stamp_arrow, straight_alpha};

use super::Drag;
use crate::app::graphics::Graphics;
use crate::app::running::Running;

const ATLAS_ATTEMPTS: usize = 2;

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

#[derive(Clone, Copy)]
struct GhostTab<'a> {
    theme: &'a Theme,
    metrics: UiMetrics,
    label: &'a str,
}

trait GhostGraphics {
    fn take_fresh(&mut self) -> bool;
    fn render_image(&mut self, scene: &Scene, atlas: &mut Atlas) -> Result<Vec<u8>, ReadbackError>;
}

impl GhostGraphics for Graphics {
    fn take_fresh(&mut self) -> bool {
        self.take_fresh()
    }

    fn render_image(&mut self, scene: &Scene, atlas: &mut Atlas) -> Result<Vec<u8>, ReadbackError> {
        self.render_image(scene, atlas)
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
        let tab = GhostTab {
            theme: &self.desk.theme,
            metrics: self.metrics,
            label,
        };
        let (pixels, width, height) =
            ghost_pixels_with_retry(&mut self.graphics, &mut self.text, tab)?;
        let source = CustomCursor::from_rgba(pixels, width, height, 0, 0)?;
        Ok(event_loop.create_custom_cursor(source))
    }
}

fn ghost_pixels_with_retry(
    graphics: &mut impl GhostGraphics,
    text: &mut TextSystem,
    tab: GhostTab<'_>,
) -> Result<(Vec<u8>, u16, u16), ReadbackError> {
    let fresh = graphics.take_fresh();
    let result = ghost_cursor_pixels(fresh, text, tab, |scene, atlas| {
        graphics.render_image(scene, atlas)
    });
    if result.is_err() && graphics.take_fresh() {
        return ghost_cursor_pixels(true, text, tab, |scene, atlas| {
            graphics.render_image(scene, atlas)
        });
    }
    result
}

fn ghost_cursor_pixels(
    fresh: bool,
    text: &mut TextSystem,
    tab: GhostTab<'_>,
    render: impl FnOnce(&Scene, &mut Atlas) -> Result<Vec<u8>, ReadbackError>,
) -> Result<(Vec<u8>, u16, u16), ReadbackError> {
    Running::sync_fresh(fresh, text, None);
    let mut scene = Scene::new(0.0, 0.0);
    for _ in 0..ATLAS_ATTEMPTS {
        scene = ghost_scene(text, tab.theme, tab.metrics, tab.label);
        if !text.end_frame() {
            break;
        }
    }
    let mut pixels = render(&scene, text.atlas_mut())?;
    let width = scene.width() as u16;
    let height = scene.height() as u16;
    straight_alpha(&mut pixels);
    stamp_arrow(&mut pixels, u32::from(width), tab.metrics);
    Ok((pixels, width, height))
}
