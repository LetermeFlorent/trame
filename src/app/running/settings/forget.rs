use trame::preferences::{SshHost, vault};

use crate::app::persist;
use crate::app::running::{Closing, Running};

impl Running {
    pub(super) fn forget_listed(&mut self, index: usize) {
        let Some(host) = self.listed_host(index) else {
            return;
        };
        if host.alias().is_some_and(written_by_hand) {
            self.closing = Some(Closing::Forget(index));
            return;
        }
        self.forget(&host);
    }

    pub(in crate::app::running) fn forget_at(&mut self, index: usize) {
        if let Some(host) = self.listed_host(index) {
            self.forget(&host);
        }
    }

    pub(in crate::app::running) fn listed_host(&mut self, index: usize) -> Option<SshHost> {
        self.panel_mut()?.hosts.get(index).cloned()
    }

    fn forget(&mut self, host: &SshHost) {
        if let (Some(alias), Some(config)) = (host.alias(), persist::ssh_config())
            && let Err(error) = config.remove(alias)
        {
            eprintln!("trame: {alias} could not be removed from the SSH config: {error}");
        }
        vault::SSH.forget(host);
        self.set_preferences(self.preferences.forget_host(host));
        self.reload_hosts();
    }
}

fn written_by_hand(alias: &str) -> bool {
    let Some(config) = persist::ssh_config() else {
        return false;
    };
    config.written_by_hand(alias).unwrap_or_else(|error| {
        eprintln!("trame: the SSH config could not be read: {error}");
        false
    })
}
