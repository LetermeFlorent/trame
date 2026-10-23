use winit::event::KeyEvent;

use trame::preferences::Preferences;
use trame::ui::frame::SettingControl;

use super::keyboard::is_escape;
use super::{FolderTarget, Running};

impl Running {
    pub(super) fn place_settings(&mut self) {
        if self.settings_open {
            let (text, preferences) = (&mut self.text, &self.preferences);
            self.frame.place_settings(text, preferences, self.strings);
        }
    }

    pub(super) fn toggle_settings(&mut self) {
        self.settings_open = !self.settings_open;
        self.changed();
    }

    pub(super) fn settings_key(&mut self, event: &KeyEvent) -> bool {
        if !self.settings_open {
            return false;
        }
        if is_escape(event) {
            self.toggle_settings();
        }
        true
    }

    pub(super) fn apply_setting(&mut self, control: SettingControl) {
        let current = &self.preferences;
        let next = match control {
            SettingControl::Close => return self.toggle_settings(),
            SettingControl::RestoreText => current.toggle_restore_text(),
            SettingControl::TextSmaller => current.step_font_size(-1),
            SettingControl::TextBigger => current.step_font_size(1),
            SettingControl::InterfaceSmaller => current.step_interface_size(-1),
            SettingControl::InterfaceBigger => current.step_interface_size(1),
            SettingControl::Language(language) => Preferences {
                language,
                ..current.clone()
            },
            SettingControl::DefaultFolder => return self.ask_folder(FolderTarget::Default),
            SettingControl::ClearFolder => Preferences {
                default_folder: None,
                ..current.clone()
            },
            SettingControl::ForgetHost(index) => current.forget_host(index),
        };
        self.set_preferences(next);
    }
}
