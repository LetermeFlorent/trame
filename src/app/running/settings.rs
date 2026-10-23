use winit::event::KeyEvent;

use trame::input::TextField;
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
        if let Some(field) = &self.command_field {
            self.frame.place_command_field(&mut self.text, field);
        }
    }

    pub(super) fn toggle_settings(&mut self) {
        self.settings_open = !self.settings_open;
        self.command_field = None;
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
            SettingControl::Recall(recall) => current.toggle(recall),
            SettingControl::StartupCommand => return self.edit_command(),
            SettingControl::ClearStartupCommand => current.with_startup_command(""),
            SettingControl::InstanceTabsFewer => current.step_instance_tabs(-1),
            SettingControl::InstanceTabsMore => current.step_instance_tabs(1),
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

    fn edit_command(&mut self) {
        let command = self.preferences.startup_command.as_deref();
        self.command_field = Some(TextField::new(command.unwrap_or_default()));
        self.changed();
    }
}
