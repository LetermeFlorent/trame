mod config;

use crate::host::persist;
use crate::preferences::{ConnectForm, HostEntry, SshHost, vault};

use super::{Closing, Desk, Effect, Effects, Outcome};
use config::{changed_in_config, kept_in_config, save_secret};

pub use config::written_by_hand;

pub struct HostForm {
    pub form: ConnectForm,
    pub original: Option<SshHost>,
}

impl HostForm {
    pub fn new() -> Self {
        Self {
            form: ConnectForm::new(),
            original: None,
        }
    }

    pub fn changing(host: SshHost) -> Self {
        let entry = host.alias().and_then(persist::ssh_entry);
        let entry = entry.unwrap_or_else(|| HostEntry::of(&host));
        let secret = vault::SSH.load(&host);
        Self {
            form: ConnectForm::filled(&entry, secret.as_deref()),
            original: Some(host),
        }
    }
}

impl Default for HostForm {
    fn default() -> Self {
        Self::new()
    }
}

impl Desk {
    pub fn store_host(&mut self, entry: &mut HostForm) -> Outcome<Option<SshHost>> {
        let Some(host) = entry.form.host() else {
            if let Some(field) = entry.form.first_invalid() {
                entry.form.focus_on(field);
            }
            return Outcome {
                value: None,
                effects: Effects::default(),
            };
        };
        let secret = entry.form.secret();
        let (host, effects) = match &entry.original {
            Some(original) => {
                let host = changed_in_config(original, host);
                if *original != host || secret.is_none() {
                    vault::SSH.forget(original);
                }
                let next = self.preferences.replace_host(original, host.clone());
                (host, self.store_preferences(next))
            }
            None => {
                let host = kept_in_config(host);
                let next = self.preferences.remember_host(host.clone());
                (host, self.store_preferences(next))
            }
        };
        save_secret(&host, secret);
        Outcome {
            value: Some(host),
            effects,
        }
    }

    pub fn ask_forget(&mut self, host: SshHost) -> Effects {
        if written_by_hand(&host) {
            self.closing = Some(Closing::Forget(host));
            return Effect::Changed.into();
        }
        let mut effects = self.forget_host(&host);
        effects.push(Effect::HostsChanged);
        effects
    }

    pub fn forget_host(&mut self, host: &SshHost) -> Effects {
        config::remove_from_config(host);
        vault::SSH.forget(host);
        self.store_preferences(self.preferences.forget_host(host))
    }
}
