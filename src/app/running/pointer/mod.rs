mod cursor;
mod drag;
mod files;
mod press;
pub(super) mod scroll;
pub(in crate::app::running) mod selection;

use std::time::{Duration, Instant};

use winit::event::{ElementState, MouseButton};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Cursor, CursorIcon};

use trame::geometry::Point;
use trame::ui::Hit;

use super::Running;
pub(super) use cursor::divider_cursor;
use cursor::hover_cursor_for;
pub(super) use drag::{Drag, Ghost};
pub(super) use scroll::Scroll;
use selection::LinkHover;
pub(super) use selection::{HoverPreview, Selecting};
const MULTI_CLICK: Duration = Duration::from_millis(500);

#[derive(Default)]
pub(super) struct Pointer {
    pub position: Option<Point>,
    pub pressed: Option<Hit>,
    pub last_click: Option<(Instant, Hit)>,
    pub cursor: Cursor,
    pub ghost: Ghost,
    pub preview: Option<HoverPreview>,
    pub wheel: f32,
}

impl Running {
    pub(super) fn pointer_moved(&mut self, event_loop: &ActiveEventLoop, point: Point) {
        self.pointer.position = Some(point);
        if self.drag_moved(point) {
            self.lift_ghost(event_loop);
            return;
        }
        self.refresh_hover();
        self.terminal_motion(point);
    }

    pub(in crate::app::running) fn drop_press(&mut self) {
        self.pointer.pressed = None;
        self.view.pressed = None;
    }

    pub(super) fn pointer_left(&mut self) {
        self.pointer.position = None;
        self.refresh_hover();
    }

    pub(super) fn refresh_hover(&mut self) {
        if self.picture_active() {
            self.clear_hover_preview();
            return self.refresh_picture_hover();
        }
        let resizable = self.resizable();
        let hover = self
            .pointer
            .position
            .filter(|_| !self.drags_tab())
            .and_then(|point| self.frame.hit(point, resizable));
        if hover != self.view.hover {
            self.view.hover = hover;
            self.window.request_redraw();
        }
        let drag = self.drag_cursor();
        let link = drag.is_none().then(|| self.hovered_link(hover)).flatten();
        let has_link = link.is_some();
        let image = match link {
            Some(LinkHover::Image(path, anchor)) => Some((path, anchor)),
            _ => None,
        };
        self.refresh_hover_preview(image);
        let cursor = drag.unwrap_or_else(|| Cursor::Icon(self.hover_cursor(hover, has_link)));
        if cursor != self.pointer.cursor {
            self.pointer.cursor = cursor.clone();
            self.window.set_cursor(cursor);
        }
    }

    pub(super) fn mouse_button(&mut self, exiting: bool, state: ElementState, button: MouseButton) {
        if self.closing_button(state)
            || self.picture_button(state, button)
            || self.menu_button(exiting, state, button)
            || self.editing_button(state)
            || self.terminal_button(state, button)
        {
            return;
        }
        let hit = self
            .pointer
            .position
            .and_then(|point| self.frame.hit(point, self.resizable()));
        match (button, state) {
            (MouseButton::Left, ElementState::Pressed) => self.press(exiting, hit),
            (MouseButton::Left, ElementState::Released) => self.release(exiting, hit),
            (MouseButton::Middle, ElementState::Released) => self.middle_click(exiting, hit),
            _ => {}
        }
    }

    pub(super) fn resizable(&self) -> bool {
        self.metrics.platform.draws_window_controls() && !self.window.is_maximized()
    }

    fn hover_cursor(&self, hover: Option<Hit>, link: bool) -> CursorIcon {
        let clickable = match (hover, self.pointer.position) {
            (Some(Hit::Terminal(pane)), Some(point)) => link || self.picture_under(pane, point),
            _ => false,
        };
        hover_cursor_for(hover, &self.frame.dividers, clickable)
    }
}
