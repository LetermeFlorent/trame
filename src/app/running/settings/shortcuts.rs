use winit::event::KeyEvent;
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;

use trame::input::{Combo, Overrides, Pressed, pressed};
use trame::ui::frame::{KeyControl, KeyNotice, ListControl, ShortcutsView};

use crate::app::running::Running;
use crate::app::running::keys::{is_escape, shortcut_input};

impl Running {
    pub(in crate::app::running) fn keys_view(&mut self) -> Option<&mut ShortcutsView> {
        self.settings.as_mut()?.keys.as_mut()
    }

    pub(super) fn capturing_key(&self) -> bool {
        let view = self.settings.as_ref().and_then(|state| state.keys);
        view.is_some_and(|view| view.capturing)
    }

    pub(super) fn key_setting(&mut self, control: KeyControl) {
        let Some(view) = self.keys_view() else {
            return;
        };
        view.notice = None;
        let action = view.action;
        match (control, action) {
            (KeyControl::Back, _) => {
                *view = ShortcutsView {
                    first: view.first,
                    ..ShortcutsView::default()
                };
            }
            (KeyControl::Add, Some(_)) => view.capturing = true,
            (KeyControl::ResetAll, _) => self.set_shortcuts(Overrides::default()),
            (KeyControl::Reset, Some(action)) => {
                let next = self.preferences.shortcuts.reset(action);
                self.set_shortcuts(next);
            }
            (KeyControl::Remove(index), Some(action)) => {
                let mut keys = self.keymap().keys(action);
                if index < keys.len() {
                    keys.remove(index);
                }
                self.set_keys(action, keys);
            }
            _ => {}
        }
        self.changed();
    }

    pub(in crate::app::running) fn shortcut_control(&mut self, control: ListControl) -> bool {
        if self.editing.is_some() {
            return false;
        }
        let Some(view) = self.keys_view() else {
            return false;
        };
        match control {
            ListControl::Pick(action) | ListControl::Change(action) => {
                *view = ShortcutsView {
                    first: view.first,
                    action: Some(action),
                    ..ShortcutsView::default()
                };
                true
            }
            ListControl::Forget(_) => true,
            ListControl::Thumb | ListControl::Track => false,
        }
    }

    pub(super) fn capture_key(&mut self, event: &KeyEvent) {
        if is_escape(event) && self.modifiers.is_empty() {
            return self.stop_capture();
        }
        let base = event.key_without_modifiers();
        let input = shortcut_input(event, &base, self.modifiers);
        let combo = match pressed(input, self.metrics.platform) {
            Pressed::Modifier => return,
            Pressed::Typing => return self.key_notice(KeyNotice::Typing),
            Pressed::Unsupported => return self.key_notice(KeyNotice::Unsupported),
            Pressed::Combo(combo) => combo,
        };
        let Some(action) = self.keys_view().and_then(|view| view.action) else {
            return;
        };
        let keymap = self.keymap();
        match keymap.owner(&combo).or_else(|| keymap.action_for(input)) {
            Some(owner) if owner != action => self.key_notice(KeyNotice::Taken(owner)),
            Some(_) => self.stop_capture(),
            None => {
                let mut keys = keymap.keys(action);
                keys.push(combo);
                self.set_keys(action, keys);
                self.stop_capture();
            }
        }
    }

    fn stop_capture(&mut self) {
        if let Some(view) = self.keys_view() {
            view.capturing = false;
            view.notice = None;
        }
        self.changed();
    }

    fn key_notice(&mut self, notice: KeyNotice) {
        if let Some(view) = self.keys_view() {
            view.notice = Some(notice);
        }
        self.changed();
    }

    fn set_keys(&mut self, action: usize, keys: Vec<Combo>) {
        let platform = self.metrics.platform;
        let next = self.preferences.shortcuts.with_keys(action, keys, platform);
        self.set_shortcuts(next);
    }

    fn set_shortcuts(&mut self, shortcuts: Overrides) {
        let next = self.preferences.with(|next| next.shortcuts = shortcuts);
        self.set_preferences(next);
    }
}
