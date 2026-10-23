mod drag;
mod press;
mod scroll;
mod selection;

use std::time::Instant;

use winit::event::{ElementState, MouseButton};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Cursor, CursorIcon};

use trame::geometry::Point;
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Axis;

use super::Running;
use crate::app::window;
pub(super) use drag::{Drag, Ghost};
pub(super) use scroll::Scroll;
pub(super) use selection::Selecting;

#[derive(Default)]
pub(super) struct Pointer {
    pub position: Option<Point>,
    pub pressed: Option<Hit>,
    pub last_click: Option<(Instant, Hit)>,
    pub cursor: Cursor,
    pub ghost: Ghost,
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

    pub(super) fn pointer_left(&mut self) {
        self.pointer.position = None;
        self.refresh_hover();
    }

    pub(super) fn refresh_hover(&mut self) {
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
        let cursor = self
            .drag_cursor()
            .unwrap_or_else(|| Cursor::Icon(cursor_for(hover, &self.frame)));
        if cursor != self.pointer.cursor {
            self.pointer.cursor = cursor.clone();
            self.window.set_cursor(cursor);
        }
    }

    pub(super) fn mouse_button(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
        button: MouseButton,
    ) {
        if self.closing_button(event_loop, state)
            || self.menu_button(state, button)
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
            (MouseButton::Left, ElementState::Pressed) => self.press(event_loop, hit),
            (MouseButton::Left, ElementState::Released) => self.release(event_loop, hit),
            (MouseButton::Middle, ElementState::Released) => self.middle_click(event_loop, hit),
            _ => {}
        }
    }

    pub(super) fn resizable(&self) -> bool {
        self.metrics.platform.draws_window_controls() && !self.window.is_maximized()
    }
}

fn cursor_for(hit: Option<Hit>, frame: &FrameLayout) -> CursorIcon {
    match hit {
        Some(Hit::Resize(edge)) => window::resize_cursor(edge),
        Some(Hit::Divider(index)) => match frame.dividers.get(index).map(|divider| divider.axis) {
            Some(Axis::Row) => CursorIcon::ColResize,
            Some(Axis::Column) => CursorIcon::RowResize,
            None => CursorIcon::Default,
        },
        Some(Hit::SidebarEdge) => CursorIcon::ColResize,
        Some(Hit::Terminal(_) | Hit::EditField | Hit::FormField(_)) => CursorIcon::Text,
        _ => CursorIcon::Default,
    }
}
