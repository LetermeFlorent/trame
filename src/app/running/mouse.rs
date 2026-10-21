use std::time::{Duration, Instant};

use winit::event::{ElementState, MouseButton};
use winit::event_loop::ActiveEventLoop;
use winit::window::CursorIcon;

use trame::geometry::Point;
use trame::input::Command;
use trame::ui::frame::SidebarItem;
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Axis;

use super::Running;
use crate::app::window;

const DOUBLE_CLICK: Duration = Duration::from_millis(500);

#[derive(Default)]
pub(super) struct Pointer {
    pub position: Option<Point>,
    pub pressed: Option<Hit>,
    pub last_click: Option<(Instant, Hit)>,
    pub cursor: CursorIcon,
    pub wheel: f32,
}

impl Running {
    pub(super) fn pointer_moved(&mut self, point: Point) {
        self.pointer.position = Some(point);
        self.refresh_hover();
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
            .and_then(|point| self.frame.hit(point, resizable));
        if hover != self.view.hover {
            self.view.hover = hover;
            self.window.request_redraw();
        }
        let cursor = cursor_for(hover, &self.frame);
        if cursor != self.pointer.cursor {
            self.pointer.cursor = cursor;
            self.window.set_cursor(cursor);
        }
    }

    pub(super) fn mouse_button(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
        button: MouseButton,
    ) {
        let hit = self
            .pointer
            .position
            .and_then(|point| self.frame.hit(point, self.resizable()));
        match (button, state) {
            (MouseButton::Left, ElementState::Pressed) => self.press(hit),
            (MouseButton::Left, ElementState::Released) => self.release(event_loop, hit),
            (MouseButton::Middle, ElementState::Released) => self.middle_click(event_loop, hit),
            _ => {}
        }
    }

    fn press(&mut self, hit: Option<Hit>) {
        let Some(hit) = hit else {
            return;
        };
        let previous = self.pointer.last_click.replace((Instant::now(), hit));
        let double = previous.is_some_and(|(at, last)| last == hit && at.elapsed() < DOUBLE_CLICK);
        match hit {
            Hit::Resize(edge) => {
                let direction = window::resize_direction(edge);
                report(self.window.drag_resize_window(direction), "resize");
                return;
            }
            Hit::TitleBar if double => {
                self.pointer.last_click = None;
                self.toggle_maximized();
                return;
            }
            Hit::TitleBar => {
                report(self.window.drag_window(), "move");
                return;
            }
            Hit::Tab(_, id) | Hit::Row(SidebarItem::Tab(id)) => self.workspace.activate_tab(id),
            Hit::Row(SidebarItem::Instance(instance)) => self.activate_instance(instance),
            Hit::Terminal(pane) | Hit::TabBar(pane) => self.focus_pane(pane),
            _ => {}
        }
        self.pointer.pressed = Some(hit);
        self.view.pressed = Some(hit);
        self.changed();
    }

    fn release(&mut self, event_loop: &ActiveEventLoop, hit: Option<Hit>) {
        self.view.pressed = None;
        let pressed = self.pointer.pressed.take();
        if let Some(pressed) = pressed.filter(|pressed| hit == Some(*pressed)) {
            self.click(event_loop, pressed);
        }
        self.changed();
    }

    fn click(&mut self, event_loop: &ActiveEventLoop, hit: Hit) {
        match hit {
            Hit::SidebarToggle => self.command(event_loop, Command::ToggleSidebar),
            Hit::Minimize => self.window.set_minimized(true),
            Hit::Maximize => self.toggle_maximized(),
            Hit::Close => event_loop.exit(),
            Hit::NewInstance => self.command(event_loop, Command::NewInstance),
            Hit::RowClose(SidebarItem::Instance(id)) => self.close_instance(event_loop, id),
            Hit::RowClose(SidebarItem::Tab(id)) | Hit::TabClose(_, id) => {
                self.close_tab(event_loop, id)
            }
            Hit::NewTab(pane) => {
                self.focus_pane(pane);
                self.command(event_loop, Command::NewTab);
            }
            Hit::Split(pane) => {
                self.focus_pane(pane);
                let command = match self.modifiers.shift_key() {
                    true => Command::SplitDown,
                    false => Command::SplitRight,
                };
                self.command(event_loop, command);
            }
            _ => {}
        }
    }

    fn middle_click(&mut self, event_loop: &ActiveEventLoop, hit: Option<Hit>) {
        if let Some(Hit::Tab(_, id) | Hit::Row(SidebarItem::Tab(id))) = hit {
            self.close_tab(event_loop, id);
        }
    }

    fn toggle_maximized(&mut self) {
        self.window.set_maximized(!self.window.is_maximized());
    }

    fn resizable(&self) -> bool {
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
        Some(Hit::Terminal(_)) => CursorIcon::Text,
        _ => CursorIcon::Default,
    }
}

fn report(result: Result<(), winit::error::ExternalError>, action: &str) {
    if let Err(error) = result {
        eprintln!("trame: the window could not {action}: {error}");
    }
}
