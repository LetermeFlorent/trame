mod drag;
mod files;
mod press;
mod scroll;
mod selection;

use std::time::{Duration, Instant};

use winit::event::{ElementState, MouseButton};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Cursor, CursorIcon};

use trame::geometry::Point;
use trame::ui::frame::PickerPart;
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Axis;

use super::Running;
use crate::app::window;
pub(super) use drag::{Drag, Ghost};
pub(super) use scroll::Scroll;
pub(super) use selection::Selecting;
const MULTI_CLICK: Duration = Duration::from_millis(500);

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

    pub(in crate::app::running) fn drop_press(&mut self) {
        self.pointer.pressed = None;
        self.view.pressed = None;
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
            .unwrap_or_else(|| Cursor::Icon(self.hover_cursor(hover)));
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
            || self.menu_button(event_loop, state, button)
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

    fn hover_cursor(&self, hover: Option<Hit>) -> CursorIcon {
        let on_link = match (hover, self.pointer.position) {
            (Some(Hit::Terminal(pane)), Some(point)) => self.link_under(pane, point),
            _ => false,
        };
        if on_link {
            CursorIcon::Pointer
        } else {
            cursor_for(hover, &self.frame)
        }
    }
}

fn cursor_for(hit: Option<Hit>, frame: &FrameLayout) -> CursorIcon {
    match hit {
        Some(Hit::Resize(edge)) => window::resize_cursor(edge),
        Some(Hit::Divider(index)) => frame
            .dividers
            .get(index)
            .map_or(CursorIcon::Default, |divider| divider_cursor(divider.axis)),
        Some(Hit::SidebarEdge) => CursorIcon::ColResize,
        Some(
            Hit::Terminal(_) | Hit::EditField | Hit::FormField(_) | Hit::Picker(PickerPart::Field),
        ) => CursorIcon::Text,
        None
        | Some(
            Hit::TitleBar
            | Hit::Sidebar
            | Hit::TabBar(_)
            | Hit::EditStrip
            | Hit::SettingsPanel
            | Hit::Menu
            | Hit::Dialog
            | Hit::Backdrop,
        ) => CursorIcon::Default,
        Some(_) => CursorIcon::Pointer,
    }
}

fn divider_cursor(axis: Axis) -> CursorIcon {
    match axis {
        Axis::Row => CursorIcon::ColResize,
        Axis::Column => CursorIcon::RowResize,
    }
}
