use trame::preferences::{ConnectForm, SshHost};
use trame::ui::frame::HostsView;

use super::SettingsState;
use super::store::HostForm;
use crate::app::running::Running;

pub(in crate::app::running) struct HostsPanel {
    pub(in crate::app::running) hosts: Vec<SshHost>,
    pub(in crate::app::running) first: usize,
    pub(in crate::app::running) form: Option<HostForm>,
}

impl HostsPanel {
    pub(in crate::app::running) fn new(hosts: Vec<SshHost>) -> Self {
        Self {
            hosts,
            first: 0,
            form: None,
        }
    }

    pub(in crate::app::running) fn connecting(hosts: Vec<SshHost>) -> Self {
        Self {
            form: Some(HostForm::new()),
            ..Self::new(hosts)
        }
    }

    pub(in crate::app::running) fn view(&self) -> HostsView<'_> {
        let entry = self.form.as_ref();
        HostsView {
            hosts: &self.hosts,
            first: self.first,
            form: entry.map(|entry| &entry.form),
            changing: entry.is_some_and(|entry| entry.original.is_some()),
        }
    }
}

pub(super) fn settings_panel(settings: &mut Option<SettingsState>) -> Option<&mut HostsPanel> {
    settings.as_mut()?.hosts.as_mut()
}

impl Running {
    pub(in crate::app::running) fn panel_mut(&mut self) -> Option<&mut HostsPanel> {
        settings_panel(&mut self.settings).or_else(|| self.editing.as_mut()?.panel_mut())
    }

    pub(in crate::app::running) fn connect_form_mut(&mut self) -> Option<&mut ConnectForm> {
        let entry = self.panel_mut()?.form.as_mut()?;
        Some(&mut entry.form)
    }

    pub(in crate::app::running) fn settings_form_open(&self) -> bool {
        let hosts = self
            .settings
            .as_ref()
            .and_then(|state| state.hosts.as_ref());
        hosts.is_some_and(|panel| panel.form.is_some())
    }

    pub(in crate::app::running) fn close_settings_form(&mut self) -> bool {
        settings_panel(&mut self.settings).is_some_and(|panel| panel.form.take().is_some())
    }

    pub(super) fn add_host(&mut self) {
        if let Some(panel) = settings_panel(&mut self.settings) {
            panel.form = Some(HostForm::new());
        }
        self.changed();
    }

    pub(in crate::app::running) fn reload_hosts(&mut self) {
        let hosts = self.known_hosts();
        if let Some(panel) = self.panel_mut() {
            panel.hosts = hosts;
        }
        self.changed();
    }
}
