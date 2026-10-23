use trame::preferences::{ConnectForm, HostEntry, SshHost, vault};

use super::hosts::settings_panel;
use crate::app::persist;
use crate::app::running::Running;

pub(in crate::app::running) struct HostForm {
    pub(in crate::app::running) form: ConnectForm,
    pub(in crate::app::running) original: Option<SshHost>,
}

impl HostForm {
    pub(in crate::app::running) fn new() -> Self {
        Self {
            form: ConnectForm::new(),
            original: None,
        }
    }

    fn changing(host: SshHost) -> Self {
        let entry = host.alias().and_then(config_entry);
        let entry = entry.unwrap_or_else(|| HostEntry::of(&host));
        let secret = vault::SSH.load(&host);
        Self {
            form: ConnectForm::filled(&entry, secret.as_deref()),
            original: Some(host),
        }
    }
}

impl Running {
    pub(in crate::app::running) fn store_host(&mut self, entry: &mut HostForm) -> Option<SshHost> {
        let Some(host) = entry.form.host() else {
            if let Some(field) = entry.form.first_invalid() {
                entry.form.focus_on(field);
            }
            return None;
        };
        let secret = entry.form.secret();
        let host = match &entry.original {
            Some(original) => {
                let host = changed_in_config(original, host);
                if *original != host || secret.is_none() {
                    vault::SSH.forget(original);
                }
                self.set_preferences(self.preferences.replace_host(original, host.clone()));
                host
            }
            None => {
                let host = kept_in_config(host);
                self.set_preferences(self.preferences.remember_host(host.clone()));
                host
            }
        };
        save_secret(&host, secret);
        Some(host)
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

    pub(super) fn forget_listed(&mut self, index: usize) {
        let host = self
            .panel_mut()
            .and_then(|panel| panel.hosts.get(index).cloned());
        let Some(host) = host else {
            return;
        };
        if let (Some(alias), Some(config)) = (host.alias(), persist::ssh_config())
            && let Err(error) = config.remove(alias)
        {
            eprintln!("trame: {alias} could not be removed from the SSH config: {error}");
        }
        vault::SSH.forget(&host);
        self.set_preferences(self.preferences.forget_host(&host));
        self.reload_hosts();
    }
}

fn config_entry(alias: &str) -> Option<HostEntry> {
    let config = persist::ssh_config()?;
    config.entry(alias).unwrap_or_else(|error| {
        eprintln!("trame: the SSH config could not be read: {error}");
        None
    })
}

fn save_secret(host: &SshHost, secret: Option<&str>) {
    let Some(secret) = secret else {
        return;
    };
    if let Err(error) = vault::SSH.store(host, secret) {
        eprintln!(
            "trame: the password for {} could not be saved: {error}",
            host.label()
        );
    }
}

fn kept_in_config(host: SshHost) -> SshHost {
    let Some(config) = persist::ssh_config() else {
        return host;
    };
    config.keep(&host).unwrap_or_else(|error| {
        eprintln!(
            "trame: {} could not be added to the SSH config: {error}",
            host.label()
        );
        host
    })
}

fn changed_in_config(original: &SshHost, host: SshHost) -> SshHost {
    let (Some(alias), Some(config)) = (original.alias(), persist::ssh_config()) else {
        return kept_in_config(host);
    };
    config.replace(alias, &host).unwrap_or_else(|error| {
        eprintln!(
            "trame: {} could not be changed in the SSH config: {error}",
            host.label()
        );
        host
    })
}
