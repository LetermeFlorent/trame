use winit::event::{ElementState, KeyEvent};

use trame::ui::Hit;
use trame::ui::frame::SettingControl;

use super::Running;
use super::keyboard::is_escape;

impl Running {
    pub(super) fn place_settings(&mut self) {
        if self.settings_open {
            self.frame.place_settings(&mut self.text, &self.preferences);
        }
    }

    pub(super) fn toggle_settings(&mut self) {
        self.settings_open = !self.settings_open;
        self.changed();
    }

    pub(super) fn dismiss_settings(&mut self, state: ElementState) {
        if !self.settings_open || state != ElementState::Pressed {
            return;
        }
        let hit = self
            .pointer
            .position
            .and_then(|point| self.frame.hit(point, false));
        let inside = matches!(
            hit,
            Some(Hit::Settings | Hit::SettingsPanel | Hit::Setting(_))
        );
        if !inside {
            self.toggle_settings();
        }
    }

    pub(super) fn settings_key(&mut self, event: &KeyEvent) -> bool {
        if !self.settings_open || !is_escape(event) {
            return false;
        }
        self.toggle_settings();
        true
    }

    pub(super) fn apply_setting(&mut self, control: SettingControl) {
        let current = self.preferences;
        let next = match control {
            SettingControl::RestoreText => current.toggle_restore_text(),
            SettingControl::TextSmaller => current.step_font_size(-1),
            SettingControl::TextBigger => current.step_font_size(1),
            SettingControl::InterfaceSmaller => current.step_interface_size(-1),
            SettingControl::InterfaceBigger => current.step_interface_size(1),
        };
        self.set_preferences(next);
    }
}
