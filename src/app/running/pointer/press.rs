use std::time::{Duration, Instant};

use winit::event_loop::ActiveEventLoop;

use trame::input::Command;
use trame::ui::Hit;
use trame::ui::frame::SidebarItem;

use crate::app::running::Running;
use crate::app::window;

const DOUBLE_CLICK: Duration = Duration::from_millis(500);

impl Running {
    pub(super) fn press(&mut self, event_loop: &ActiveEventLoop, hit: Option<Hit>) {
        let Some(hit) = hit else {
            return;
        };
        let previous = self.pointer.last_click.replace((Instant::now(), hit));
        let double = previous.is_some_and(|(at, last)| last == hit && at.elapsed() < DOUBLE_CLICK);
        if double && self.double_press(event_loop, hit) {
            self.pointer.last_click = None;
            return;
        }
        match hit {
            Hit::Resize(edge) => {
                let direction = window::resize_direction(edge);
                return report(self.window.drag_resize_window(direction), "resize");
            }
            Hit::TitleBar => return report(self.window.drag_window(), "move"),
            Hit::Tab(_, id) | Hit::Row(SidebarItem::Tab(id)) => self.workspace.activate_tab(id),
            Hit::Row(SidebarItem::Instance(instance)) | Hit::RailInstance(instance) => {
                self.activate_instance(instance)
            }
            Hit::Terminal(pane) | Hit::TabBar(pane) => self.focus_pane(pane),
            _ => {}
        }
        self.drag_pressed(hit);
        self.pointer.pressed = Some(hit);
        self.view.pressed = Some(hit);
        self.changed();
    }

    fn double_press(&mut self, event_loop: &ActiveEventLoop, hit: Hit) -> bool {
        match hit {
            Hit::TitleBar => self.toggle_maximized(),
            Hit::TabBar(pane) => {
                self.focus_pane(pane);
                self.command(event_loop, Command::NewTab);
            }
            _ => return self.open_editor(hit),
        }
        true
    }

    pub(super) fn release(&mut self, event_loop: &ActiveEventLoop, hit: Option<Hit>) {
        self.view.pressed = None;
        let pressed = self.pointer.pressed.take();
        if self.drag_released() {
            self.changed();
            return;
        }
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
            Hit::Settings => self.toggle_settings(),
            Hit::Update => self.restart_for_update(event_loop),
            Hit::Setting(control) => self.apply_setting(control),
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

    pub(super) fn middle_click(&mut self, event_loop: &ActiveEventLoop, hit: Option<Hit>) {
        if let Some(Hit::Tab(_, id) | Hit::Row(SidebarItem::Tab(id))) = hit {
            self.close_tab(event_loop, id);
        }
    }

    fn toggle_maximized(&mut self) {
        self.window.set_maximized(!self.window.is_maximized());
    }
}

fn report(result: Result<(), winit::error::ExternalError>, action: &str) {
    if let Err(error) = result {
        eprintln!("trame: the window could not {action}: {error}");
    }
}
