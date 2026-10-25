mod hosts;
mod list;
mod store;

use winit::event::KeyEvent;

use trame::input::TextField;
use trame::ui::frame::{SettingControl, SettingsRequest, SettingsTab};

use super::keys::is_escape;
use super::{FolderTarget, Running};
pub(super) use hosts::HostsPanel;

#[derive(Default)]
pub(super) struct SettingsState {
    tab: SettingsTab,
    hosts: Option<HostsPanel>,
}

impl Running {
    pub(super) fn place_settings(&mut self) {
        if let Some(state) = &self.settings {
            let folder = self.active_folder();
            let request = SettingsRequest {
                preferences: &self.preferences,
                strings: self.strings,
                folder: folder.as_deref(),
                tab: state.tab,
                hosts: state.hosts.as_ref().map(HostsPanel::view),
            };
            self.frame.place_settings(&mut self.text, &request);
        }
        if let Some(field) = &self.command_field {
            self.frame.place_command_field(&mut self.text, field);
        }
    }

    pub(super) fn toggle_settings(&mut self) {
        self.settings = self.settings.is_none().then(SettingsState::default);
        self.command_field = None;
        self.changed();
    }

    pub(super) fn settings_key(&mut self, event: &KeyEvent) -> bool {
        if self.settings.is_none() {
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
            SettingControl::Tab(tab) => return self.show_tab(tab),
            SettingControl::AddHost => return self.add_host(),
            SettingControl::Recall(recall) => current.toggle(recall),
            SettingControl::ConfirmClose => current.toggle_confirm_close(),
            SettingControl::Efficiency => current.with(|next| next.efficiency = !next.efficiency),
            SettingControl::Acrylic => current.with(|next| next.acrylic = !next.acrylic),
            SettingControl::StartupCommand => return self.edit_command(),
            SettingControl::ClearStartupCommand => current.with_startup_command(""),
            SettingControl::InstanceTabsFewer => current.step_instance_tabs(-1),
            SettingControl::InstanceTabsMore => current.step_instance_tabs(1),
            SettingControl::TextSmaller => current.step_font_size(-1),
            SettingControl::TextBigger => current.step_font_size(1),
            SettingControl::InterfaceSmaller => current.step_interface_size(-1),
            SettingControl::InterfaceBigger => current.step_interface_size(1),
            SettingControl::Language(language) => current.with(|next| next.language = language),
            SettingControl::PanelSide(side) => current.with(|next| next.panel_side = side),
            SettingControl::Density(density) => current.with(|next| next.density = density),
            SettingControl::DefaultFolder => {
                let start = self.preferences.default_folder.clone();
                return self.ask_folder(FolderTarget::Default, start);
            }
            SettingControl::ClearFolder => current.with(|next| next.default_folder = None),
        };
        self.set_preferences(next);
    }

    fn show_tab(&mut self, tab: SettingsTab) {
        if self.settings.as_ref().is_none_or(|state| state.tab == tab) {
            return;
        }
        let hosts = (tab == SettingsTab::Connections).then(|| HostsPanel::new(self.known_hosts()));
        self.settings = Some(SettingsState { tab, hosts });
        self.command_field = None;
        self.changed();
    }

    fn edit_command(&mut self) {
        let command = self.preferences.startup_command.as_deref();
        self.command_field = Some(TextField::new(command.unwrap_or_default()));
        self.changed();
    }
}
