use trame::input::{Combo, Overrides, Pressed, ShortcutInput, pressed};
use trame::ui::frame::{KeyControl, KeyNotice, ListControl, ShortcutsView};

use crate::gnome::state::WindowState;

impl WindowState {
    pub fn capturing_key(&self) -> bool {
        self.settings
            .as_ref()
            .and_then(|s| s.keys)
            .is_some_and(|k| k.capturing)
    }

    pub(super) fn key_setting(&mut self, control: KeyControl) {
        let Some(action) = self
            .settings
            .as_ref()
            .and_then(|s| s.keys)
            .map(|k| k.action)
        else {
            return;
        };
        if let Some(k) = self.settings.as_mut().and_then(|s| s.keys.as_mut()) {
            k.notice = None;
        }
        self.apply_key_control(control, action);
        self.changed();
    }

    fn apply_key_control(&mut self, control: KeyControl, action: Option<usize>) {
        match (control, action) {
            (KeyControl::ResetAll, _) => self.set_shortcuts(Overrides::default()),
            (KeyControl::Reset, Some(act)) => {
                self.set_shortcuts(self.desk.preferences.shortcuts.reset(act));
            }
            (KeyControl::Remove(idx), Some(act)) => self.remove_shortcut(act, idx),
            (other, _) => self.apply_keys_view(other),
        }
    }

    fn apply_keys_view(&mut self, control: KeyControl) {
        let Some(k) = self.settings.as_mut().and_then(|s| s.keys.as_mut()) else {
            return;
        };
        match control {
            KeyControl::Back => {
                *k = ShortcutsView {
                    first: k.first,
                    ..ShortcutsView::default()
                }
            }
            KeyControl::Add => k.capturing = true,
            _ => {}
        }
    }

    fn remove_shortcut(&mut self, act: usize, idx: usize) {
        let platform = self.metrics.platform;
        let mut keys = self.keymap().keys(act);
        if idx < keys.len() {
            keys.remove(idx);
        }
        let next = self
            .desk
            .preferences
            .shortcuts
            .with_keys(act, keys, platform);
        self.set_shortcuts(next);
    }

    pub(super) fn shortcut_control(&mut self, control: ListControl) -> bool {
        if self.editing.is_some() {
            return false;
        }
        let Some(view) = self.settings.as_mut().and_then(|s| s.keys.as_mut()) else {
            return false;
        };
        match control {
            ListControl::Pick(act) | ListControl::Change(act) => {
                *view = ShortcutsView {
                    first: view.first,
                    action: Some(act),
                    ..ShortcutsView::default()
                };
                true
            }
            ListControl::Forget(_) => true,
            ListControl::Thumb | ListControl::Track => false,
        }
    }

    pub fn capture_shortcut(&mut self, input: ShortcutInput, is_esc: bool) {
        if is_esc && self.modifiers.is_empty() {
            return self.stop_capture(None);
        }
        let combo = match pressed(input, self.metrics.platform) {
            Pressed::Modifier => return,
            Pressed::Typing => return self.stop_capture(Some(KeyNotice::Typing)),
            Pressed::Unsupported => return self.stop_capture(Some(KeyNotice::Unsupported)),
            Pressed::Combo(c) => c,
        };
        let Some(action) = self
            .settings
            .as_ref()
            .and_then(|s| s.keys)
            .and_then(|k| k.action)
        else {
            return;
        };
        let owner = self
            .keymap()
            .owner(&combo)
            .or_else(|| self.keymap().action_for(input));
        self.assign_combo(action, combo, owner);
    }

    fn assign_combo(&mut self, action: usize, combo: Combo, owner: Option<usize>) {
        match owner {
            Some(o) if o != action => self.stop_capture(Some(KeyNotice::Taken(o))),
            Some(_) => self.stop_capture(None),
            None => {
                let platform = self.metrics.platform;
                let mut keys: Vec<Combo> = self.keymap().keys(action);
                keys.push(combo);
                let next = self
                    .desk
                    .preferences
                    .shortcuts
                    .with_keys(action, keys, platform);
                self.set_shortcuts(next);
                self.stop_capture(None);
            }
        }
    }

    fn stop_capture(&mut self, notice: Option<KeyNotice>) {
        if let Some(k) = self.settings.as_mut().and_then(|s| s.keys.as_mut()) {
            if notice.is_none() {
                k.capturing = false;
            }
            k.notice = notice;
        }
        self.changed();
    }

    fn set_shortcuts(&mut self, shortcuts: Overrides) {
        let next = self.desk.preferences.with(|n| n.shortcuts = shortcuts);
        self.set_preferences(next);
    }
}
