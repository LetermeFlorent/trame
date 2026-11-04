use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::event_loop::ActiveEventLoop;

use trame::geometry::Point;
use trame::input::Command;
use trame::locale::Strings;
use trame::ui::Hit;
use trame::ui::frame::{EditAnchor, EditTarget, MenuEntry, MenuRequest};
use trame::workspace::PaneId;

use super::{Running, target_of};
use crate::app::running::keys::is_escape;

pub(in super::super) struct Menu {
    target: EditTarget,
    anchor: EditAnchor,
    at: Point,
    adding: bool,
}

impl Running {
    pub(in super::super) fn menu_button(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
        button: MouseButton,
    ) -> bool {
        let Some(point) = self
            .pointer
            .position
            .filter(|_| state == ElementState::Pressed)
        else {
            return false;
        };
        let hit = self.frame.hit(point, false);
        let was_open = self.menu.is_some();
        if let Some(menu) = self.menu.take() {
            self.choose(event_loop, menu, hit);
        }
        let target = hit
            .filter(|_| button == MouseButton::Right)
            .and_then(target_of);
        if let Some((target, anchor)) = target {
            self.editing = None;
            self.menu = Some(Menu {
                target,
                anchor,
                at: point,
                adding: false,
            });
        }
        let handled = was_open || target.is_some();
        if handled {
            self.changed();
        }
        handled
    }

    pub(in super::super) fn open_adding_menu(
        &mut self,
        event_loop: &ActiveEventLoop,
        pane: PaneId,
    ) {
        self.focus_pane(pane);
        let found = self
            .frame
            .panes
            .iter()
            .find(|frame| frame.id == pane)
            .and_then(|frame| Some((frame.active?, frame.adding_menu_at())));
        let Some((tab, at)) = found else {
            return self.command(event_loop, Command::NewTab);
        };
        self.editing = None;
        self.menu = Some(Menu {
            target: EditTarget::Tab(tab),
            anchor: EditAnchor::Strip(pane),
            at,
            adding: true,
        });
        self.changed();
    }

    fn choose(&mut self, event_loop: &ActiveEventLoop, menu: Menu, hit: Option<Hit>) {
        match hit {
            Some(Hit::Menu) => self.menu = Some(menu),
            Some(Hit::MenuEntry(MenuEntry::NewTab)) => {
                if let EditAnchor::Strip(pane) = menu.anchor {
                    self.focus_pane(pane);
                }
                self.command(event_loop, Command::NewTab);
            }
            Some(Hit::MenuEntry(entry)) => self.start_edit(menu.target, menu.anchor, entry),
            _ => {}
        }
    }

    pub(in super::super) fn menu_key(&mut self, event: &KeyEvent) -> bool {
        if self.menu.is_none() {
            return false;
        }
        if is_escape(event) {
            self.menu = None;
            self.changed();
        }
        true
    }

    pub(in super::super) fn place_menu(&mut self) {
        let Some(menu) = &self.menu else {
            return;
        };
        let entries = match menu.adding {
            true => MenuEntry::adding(self.strings).to_vec(),
            false => edit_entries(menu.target, self.strings),
        };
        let request = MenuRequest {
            at: menu.at,
            entries: &entries,
        };
        self.frame.place_menu(&mut self.text, &request);
    }
}

fn edit_entries(target: EditTarget, strings: &'static Strings) -> Vec<(MenuEntry, &'static str)> {
    let mut entries = vec![
        (MenuEntry::Rename, strings.rename),
        (MenuEntry::Color, strings.change_color),
    ];
    if let EditTarget::Instance(_) = target {
        entries.push((MenuEntry::Icon, strings.change_icon));
    }
    entries.push((MenuEntry::Folder, strings.change_folder));
    entries.push((MenuEntry::Ssh, strings.ssh_connection));
    entries
}
