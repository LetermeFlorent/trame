mod actions;
mod modals;

use std::time::{Duration, Instant};

use trame::geometry::Point;
use trame::input::Command;
use trame::ui::Hit;
use trame::ui::frame::SidebarItem;

use super::drag::Drag;
use super::{WindowAction, WindowState};
use modals::cursor_name;

const MULTI_CLICK: Duration = Duration::from_millis(500);

impl WindowState {
    pub fn pointer_moved(&mut self, point: Point) {
        self.pointer = Some(point);
        if self.drag_moved(point) {
            self.cursor = self.drag_cursor_name();
            return;
        }
        self.refresh_hover();
    }

    pub fn pointer_left(&mut self) {
        self.pointer = None;
        self.refresh_hover();
    }

    pub fn refresh_hover(&mut self) {
        let resizable = !self.maximized && !self.fullscreen;
        let dragging = matches!(self.drag, Some(Drag::Tab(..)));
        let hover = self
            .pointer
            .filter(|_| !dragging)
            .and_then(|p| self.frame.hit(p, resizable));
        if hover != self.view.hover {
            self.view.hover = hover;
            self.dirty = true;
        }
        self.cursor = match &self.drag {
            Some(_) => self.drag_cursor_name(),
            None => cursor_name(hover, &self.frame.dividers),
        };
    }

    pub fn mouse_button(&mut self, exiting: bool, pressed: bool, button: u32) -> bool {
        if self.closing_button(pressed)
            || self.menu_button(exiting, pressed, button)
            || self.editing_button(pressed)
        {
            return true;
        }
        let hit = self
            .pointer
            .and_then(|p| self.frame.hit(p, !self.maximized && !self.fullscreen));
        if let Some(Hit::Terminal(pane)) = hit
            && !self.modal_open()
        {
            if pressed {
                self.desk.focus_pane(pane);
                self.dirty = true;
            }
            return false;
        }
        match (button, pressed) {
            (1, true) => self.press(exiting, hit),
            (1, false) => self.release(exiting, hit),
            (2, false) => self.middle_click(exiting, hit),
            _ => {}
        }
        true
    }

    fn press(&mut self, exiting: bool, hit: Option<Hit>) {
        let Some(hit) = hit else { return };
        let prev = self.last_click.replace((Instant::now(), hit));
        let dbl = prev.is_some_and(|(at, last)| last == hit && at.elapsed() < MULTI_CLICK);
        if dbl && self.double_press(exiting, hit) {
            self.last_click = None;
            return;
        }
        match hit {
            Hit::Resize(edge) => return self.act(WindowAction::Resize(edge)),
            Hit::TitleBar => return self.act(WindowAction::Move),
            Hit::Backdrop if self.settings.is_some() => return self.toggle_settings(),
            Hit::Tab(_, id) | Hit::Row(SidebarItem::Tab(id)) => {
                self.desk.workspace.activate_tab(id);
            }
            Hit::Row(SidebarItem::Instance(inst)) | Hit::RailInstance(inst) => {
                self.desk.activate_instance(inst);
            }
            Hit::TabBar(pane) => self.desk.focus_pane(pane),
            _ => {}
        }
        self.drag_pressed(hit);
        self.pressed_hit = Some(hit);
        self.view.pressed = Some(hit);
        self.changed();
    }

    fn double_press(&mut self, exiting: bool, hit: Hit) -> bool {
        match hit {
            Hit::TitleBar => self.act(WindowAction::ToggleMaximize),
            Hit::Tab(_, tab) if self.modifiers.control_key() => {
                self.desk.workspace.toggle_even_tabs(tab);
                self.changed();
            }
            Hit::TabBar(pane) => {
                self.desk.focus_pane(pane);
                self.run_command(exiting, Command::NewTab);
            }
            _ => return self.open_editor(hit),
        }
        true
    }

    fn release(&mut self, exiting: bool, hit: Option<Hit>) {
        self.view.pressed = None;
        let pressed = self.pressed_hit.take();
        if self.drag_released() {
            return self.changed();
        }
        if let Some(p) = pressed.filter(|p| hit == Some(*p)) {
            self.click(exiting, p);
        }
        self.changed();
    }

    fn middle_click(&mut self, exiting: bool, hit: Option<Hit>) {
        if !self.picture_open
            && let Some(Hit::Tab(_, id) | Hit::Row(SidebarItem::Tab(id))) = hit
        {
            let effects = self.desk.close_tab(exiting, id);
            self.apply(effects);
        }
    }
}
