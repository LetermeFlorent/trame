mod apply;
mod forget;
mod hosts;
mod list;
mod shortcuts;
mod store;

use winit::event::KeyEvent;

use trame::input::TextField;
use trame::ui::frame::{SettingControl, SettingsRequest, SettingsTab, ShortcutsView};

use super::Running;
use super::keys::is_escape;
pub(super) use hosts::HostsPanel;

pub(super) type SettingField = (SettingControl, TextField);

#[derive(Default)]
pub(super) struct SettingsState {
    tab: SettingsTab,
    hosts: Option<HostsPanel>,
    keys: Option<ShortcutsView>,
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
                shortcuts: state.keys,
            };
            self.frame.place_settings(&mut self.text, &request);
        }
        let popup = self
            .frame
            .settings
            .as_mut()
            .map(|frame| &mut frame.popup.rect);
        let target = popup.as_ref().map(|rect| rect.height);
        if let (Some(rect), Some(height)) = (popup, self.motion.popup_height(target)) {
            rect.height = height.round();
        }
        if let Some((control, field)) = &self.setting_field {
            self.frame
                .place_setting_field(&mut self.text, *control, field);
        }
    }

    pub(super) fn toggle_settings(&mut self) {
        self.settings = self.settings.is_none().then(SettingsState::default);
        self.setting_field = None;
        self.changed();
    }

    pub(super) fn settings_key(&mut self, event: &KeyEvent) -> bool {
        if self.settings.is_none() {
            return false;
        }
        if self.capturing_key() {
            self.capture_key(event);
        } else if is_escape(event) {
            self.toggle_settings();
        }
        true
    }

    fn show_tab(&mut self, tab: SettingsTab) {
        if self.settings.as_ref().is_none_or(|state| state.tab == tab) {
            return;
        }
        let hosts = (tab == SettingsTab::Connections).then(|| HostsPanel::new(self.known_hosts()));
        let keys = (tab == SettingsTab::Shortcuts).then(ShortcutsView::default);
        self.settings = Some(SettingsState { tab, hosts, keys });
        self.setting_field = None;
        self.changed();
    }
}
