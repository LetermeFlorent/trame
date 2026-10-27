mod hosts;
mod list;
mod shortcuts;
mod store;

use winit::event::KeyEvent;

use trame::input::TextField;
use trame::preferences::Preferences;
use trame::ui::frame::{SettingControl, SettingsRequest, SettingsTab, ShortcutsView};

use super::keys::is_escape;
use super::{FolderTarget, Running};
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

    pub(super) fn apply_setting(&mut self, control: SettingControl) {
        let current = &self.preferences;
        let next = match control {
            SettingControl::Close => return self.toggle_settings(),
            SettingControl::Tab(tab) => return self.show_tab(tab),
            SettingControl::AddHost => return self.add_host(),
            SettingControl::Keys(key) => return self.key_setting(key),
            SettingControl::Recall(recall) => current.toggle(recall),
            SettingControl::ConfirmClose => current.toggle_confirm_close(),
            SettingControl::Efficiency
            | SettingControl::RestHiddenTabs
            | SettingControl::NotifyWaiting
            | SettingControl::UpdateAtLaunch
            | SettingControl::UpdateAhead => current.with(|next| flip(next, control)),
            SettingControl::Numbers(choice) => {
                current.with(|next| choice.toggle(&mut next.numbering))
            }
            SettingControl::StartupCommand | SettingControl::HistoryLines => {
                return self.edit_setting(control);
            }
            SettingControl::ClearStartupCommand => current.with_startup_command(""),
            SettingControl::InstanceTabsFewer => current.step_instance_tabs(-1),
            SettingControl::InstanceTabsMore => current.step_instance_tabs(1),
            SettingControl::HistoryFewer => current.step_history_lines(-1),
            SettingControl::HistoryMore => current.step_history_lines(1),
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
        let keys = (tab == SettingsTab::Shortcuts).then(ShortcutsView::default);
        self.settings = Some(SettingsState { tab, hosts, keys });
        self.setting_field = None;
        self.changed();
    }

    fn edit_setting(&mut self, control: SettingControl) {
        let field = match control {
            SettingControl::HistoryLines => {
                let mut field = TextField::new(&self.preferences.history_lines.to_string());
                field.select_all();
                field
            }
            _ => TextField::new(
                self.preferences
                    .startup_command
                    .as_deref()
                    .unwrap_or_default(),
            ),
        };
        self.setting_field = Some((control, field));
        self.changed();
    }
}

fn flip(next: &mut Preferences, control: SettingControl) {
    let switch = match control {
        SettingControl::Efficiency => &mut next.efficiency,
        SettingControl::RestHiddenTabs => &mut next.rest_hidden_tabs,
        SettingControl::NotifyWaiting => &mut next.notify_waiting,
        SettingControl::UpdateAtLaunch => &mut next.updates.auto,
        SettingControl::UpdateAhead => &mut next.updates.ahead,
        _ => return,
    };
    *switch = !*switch;
}
