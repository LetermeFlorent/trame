mod hosts;
mod lists;
mod prefs;
mod shortcuts;

use trame::desk::{FolderTarget, HostForm};
use trame::input::TextField;
use trame::preferences::SshHost;
use trame::ui::frame::{HostsView, SettingControl, SettingsRequest, SettingsTab, ShortcutsView};

use crate::gnome::state::{PickerJob, WindowState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind {
    Hosts,
    Sidebar,
}

pub struct HostsPanel {
    pub hosts: Vec<SshHost>,
    pub first: usize,
    pub form: Option<HostForm>,
}

impl HostsPanel {
    pub fn new(hosts: Vec<SshHost>) -> Self {
        Self {
            hosts,
            first: 0,
            form: None,
        }
    }

    pub fn connecting(hosts: Vec<SshHost>) -> Self {
        Self {
            form: Some(HostForm::new()),
            ..Self::new(hosts)
        }
    }

    pub fn view(&self) -> HostsView<'_> {
        let entry = self.form.as_ref();
        HostsView {
            hosts: &self.hosts,
            first: self.first,
            form: entry.map(|e| &e.form),
            changing: entry.is_some_and(|e| e.original.is_some()),
        }
    }
}

#[derive(Default)]
pub struct SettingsState {
    pub tab: SettingsTab,
    pub hosts: Option<HostsPanel>,
    pub keys: Option<ShortcutsView>,
}

impl WindowState {
    pub fn place_settings(&mut self) {
        if let Some(state) = &self.settings {
            let folder = self.desk.active_folder();
            let req = SettingsRequest {
                preferences: &self.desk.preferences,
                strings: self.desk.strings,
                folder: folder.as_deref(),
                tab: state.tab,
                hosts: state.hosts.as_ref().map(HostsPanel::view),
                shortcuts: state.keys,
            };
            self.frame.place_settings(&mut self.text, &req);
        }
        let popup = self.frame.settings.as_mut().map(|f| &mut f.popup.rect);
        let target = popup.as_ref().map(|r| r.height);
        if let (Some(rect), Some(h)) = (popup, self.motion.popup_height(target)) {
            rect.height = h.round();
        }
        if let Some((control, field)) = &self.setting_field {
            self.frame
                .place_setting_field(&mut self.text, *control, field);
        }
    }

    pub fn toggle_settings(&mut self) {
        self.settings = self.settings.is_none().then(SettingsState::default);
        self.setting_field = None;
        self.changed();
    }

    pub fn show_tab(&mut self, tab: SettingsTab) {
        if self.settings.as_ref().is_none_or(|s| s.tab == tab) {
            return;
        }
        let hosts =
            (tab == SettingsTab::Connections).then(|| HostsPanel::new(self.desk.known_hosts()));
        let keys = (tab == SettingsTab::Shortcuts).then(ShortcutsView::default);
        self.settings = Some(SettingsState { tab, hosts, keys });
        self.setting_field = None;
        self.changed();
    }

    pub fn apply_setting(&mut self, control: SettingControl) {
        match control {
            SettingControl::Close => return self.toggle_settings(),
            SettingControl::Tab(tab) => return self.show_tab(tab),
            SettingControl::AddHost => return self.open_add_host(),
            SettingControl::Keys(key) => return self.key_setting(key),
            SettingControl::Fill => return self.toggle_fill(),
            SettingControl::StartupCommand | SettingControl::HistoryLines => {
                return self.open_setting_field(control);
            }
            SettingControl::DefaultFolder => {
                let start = self.desk.preferences.default_folder.clone();
                self.picker_req = Some(PickerJob::Folder(FolderTarget::Default, start));
                return;
            }
            _ => {}
        }
        let next = prefs::preference_for(&self.desk.preferences, control);
        self.set_preferences(next);
    }

    fn open_add_host(&mut self) {
        if let Some(p) = self.settings.as_mut().and_then(|s| s.hosts.as_mut()) {
            p.form = Some(HostForm::new());
        }
        self.changed();
    }

    fn open_setting_field(&mut self, control: SettingControl) {
        let cur = &self.desk.preferences;
        let field = match control {
            SettingControl::HistoryLines => {
                let mut f = TextField::new(&cur.history_lines.to_string());
                f.select_all();
                f
            }
            _ => TextField::new(cur.startup_command.as_deref().unwrap_or_default()),
        };
        self.setting_field = Some((control, field));
        self.changed();
    }
}
