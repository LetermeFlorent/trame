use std::time::Instant;

use crate::desk::{Desk, Effect, Effects, Outcome};
use crate::host::persist;
use crate::log_error;
use crate::preferences::{HostEntry, SshHost, SshLogin, menu_hosts, vault};
use crate::session::SessionId;
use crate::workspace::EditTarget;

use super::Placement;

const SSH_PORT: u16 = 22;

impl Desk {
    pub fn known_hosts(&self) -> Vec<SshHost> {
        let aliases = persist::ssh_config().map_or(Ok(Vec::new()), |config| config.aliases());
        let aliases = aliases.unwrap_or_else(|error| {
            log_error!("the SSH config could not be read: {error}");
            Vec::new()
        });
        menu_hosts(&self.preferences.ssh_hosts, &aliases)
    }

    pub fn connect(&mut self, target: EditTarget, host: SshHost) -> Effects {
        match target {
            EditTarget::Tab(id) => self.workspace.activate_tab(id),
            EditTarget::Instance(id) => self.activate_instance(id),
        }
        let cwd = self.start_folder(Placement::Tab);
        let mut effects = self
            .open_in(Placement::Tab, cwd, Some(host.clone()))
            .effects;
        effects.append(self.store_preferences(self.preferences.remember_host(host)));
        effects
    }

    pub(super) fn greet(&mut self, id: SessionId, fresh: bool) {
        if let Some(host) = self.workspace.tab_host(id).cloned() {
            self.dial(id, &host);
            return;
        }
        let line = self.preferences.startup_line().filter(|_| fresh);
        if let (Some(line), Some(session)) = (line, self.sessions.get(id)) {
            session.write(line);
        }
    }

    fn dial(&mut self, id: SessionId, host: &SshHost) {
        if let Some(session) = self.sessions.get(id) {
            session.write(host.command());
        }
        let entry = persist::ssh_entry(host.split().1);
        if let Some(secret) = vault::SSH.load(host) {
            let login = SshLogin::of(host, entry.as_ref());
            self.sessions.expect_secret(id, secret, login);
        }
        let entry = entry.unwrap_or_else(|| HostEntry::of(host));
        let port = host.port.or(entry.port).unwrap_or(SSH_PORT);
        self.sessions.watch_remote(id, entry.address, port);
    }

    pub fn redial(&mut self, now: Instant) -> Outcome<Option<Instant>> {
        let due = self.sessions.redial_due(now);
        for id in &due {
            match self.workspace.tab_host(*id).cloned() {
                Some(host) => self.dial(*id, &host),
                None => self.sessions.forget_remote(*id),
            }
        }
        let effects = match due.is_empty() {
            true => Effects::default(),
            false => Effects::from([Effect::Relayout, Effect::Redraw]),
        };
        Outcome {
            value: self.sessions.remote_deadline(now),
            effects,
        }
    }

    pub fn disconnect(&mut self, id: SessionId) -> Effects {
        self.sessions.forget_remote(id);
        Effects::from([Effect::Relayout, Effect::Redraw])
    }
}
