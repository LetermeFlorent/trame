use trame::desk::HostForm;
use trame::preferences::SshHost;

use super::hosts::settings_panel;
use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn store_host(&mut self, entry: &mut HostForm) -> Option<SshHost> {
        let stored = self.desk.store_host(entry);
        self.apply(stored.effects);
        stored.value
    }

    pub(in crate::app::running) fn save_settings_form(&mut self) {
        let panel = settings_panel(&mut self.settings);
        let Some(mut entry) = panel.and_then(|panel| panel.form.take()) else {
            return;
        };
        if self.store_host(&mut entry).is_some() {
            return self.reload_hosts();
        }
        if let Some(panel) = self.panel_mut() {
            panel.form = Some(entry);
        }
    }

    pub(super) fn change_host(&mut self, index: usize) {
        let Some(panel) = self.panel_mut() else {
            return;
        };
        if let Some(host) = panel.hosts.get(index).cloned() {
            panel.form = Some(HostForm::changing(host));
        }
    }
}
