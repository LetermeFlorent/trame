use winit::event::KeyEvent;

use trame::input::TextField;
use trame::preferences::{Preferences, vault};
use trame::ui::frame::SettingControl;

use super::keys::is_escape;
use super::{FolderTarget, Running};

impl Running {
    pub(super) fn place_settings(&mut self) {
        if self.settings_open {
            let folder = self.active_folder();
            let (text, preferences) = (&mut self.text, &self.preferences);
            let strings = self.strings;
            self.frame
                .place_settings(text, preferences, strings, folder.as_deref());
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
            SettingControl::Language(language) => current.with(|next| next.language = language),
            SettingControl::DefaultFolder => return self.ask_folder(FolderTarget::Default),
            SettingControl::ClearFolder => current.with(|next| next.default_folder = None),
            SettingControl::ForgetHost(index) => self.forget_host(index),
        };
        self.set_preferences(next);
    }

    fn forget_host(&self, index: usize) -> Preferences {
        if let Some(host) = self.preferences.ssh_hosts.get(index) {
            vault::SSH.forget(host);
        }
        self.preferences.forget_host(index)
    }

    fn edit_command(&mut self) {
        let command = self.preferences.startup_command.as_deref();
        self.command_field = Some(TextField::new(command.unwrap_or_default()));
        self.changed();
    }
}
