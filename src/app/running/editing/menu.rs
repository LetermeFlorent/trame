use winit::event::{ElementState, KeyEvent, MouseButton};

use trame::geometry::Point;
use trame::ui::Hit;
use trame::ui::frame::{EditAnchor, EditTarget, MenuEntry, MenuRequest};

use super::{Running, target_of};
use crate::app::running::keyboard::is_escape;

pub(in super::super) struct Menu {
    target: EditTarget,
    anchor: EditAnchor,
    at: Point,
}

impl Running {
    pub(in super::super) fn menu_button(
        &mut self,
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
            self.choose(menu, hit);
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
            });
        }
        let handled = was_open || target.is_some();
        if handled {
            self.changed();
        }
        handled
    }

    fn choose(&mut self, menu: Menu, hit: Option<Hit>) {
        match hit {
            Some(Hit::Menu) => self.menu = Some(menu),
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
        let strings = self.strings;
        let hosts: Vec<String> = self
            .preferences
            .ssh_hosts
            .iter()
            .map(|host| format!("SSH {}", host.label()))
            .collect();
        let mut entries = vec![
            (MenuEntry::Rename, strings.rename),
            (MenuEntry::Color, strings.change_color),
        ];
        if let EditTarget::Instance(_) = menu.target {
            entries.push((MenuEntry::Icon, strings.change_icon));
        }
        entries.push((MenuEntry::Folder, strings.change_folder));
        let saved = hosts.iter().enumerate();
        entries.extend(saved.map(|(index, label)| (MenuEntry::Host(index), label.as_str())));
        entries.push((MenuEntry::NewConnection, strings.new_connection));
        let request = MenuRequest {
            at: menu.at,
            entries: &entries,
        };
        self.frame.place_menu(&mut self.text, &request);
    }
}
