use std::time::Instant;

use trame::host::persist;
use trame::log_error;
use trame::preferences::{HostEntry, SshHost, SshLogin, menu_hosts, vault};
use trame::session::SessionId;
use trame::ui::frame::EditTarget;

use super::Placement;
use crate::app::running::Running;

const SSH_PORT: u16 = 22;

impl Running {
    pub(in crate::app::running) fn known_hosts(&self) -> Vec<SshHost> {
        let aliases = persist::ssh_config().map_or(Ok(Vec::new()), |config| config.aliases());
        let aliases = aliases.unwrap_or_else(|error| {
            log_error!("the SSH config could not be read: {error}");
            Vec::new()
        });
        menu_hosts(&self.desk.preferences.ssh_hosts, &aliases)
    }

    pub(in crate::app::running) fn connect(&mut self, target: EditTarget, host: SshHost) {
        match target {
            EditTarget::Tab(id) => self.desk.workspace.activate_tab(id),
            EditTarget::Instance(id) => self.activate_instance(id),
        }
        let cwd = self.start_folder(Placement::Tab);
        if let Err(error) = self.open_in(Placement::Tab, cwd.as_deref(), Some(host.clone())) {
            log_error!("the SSH tab could not start: {error}");
        }
        self.set_preferences(self.desk.preferences.remember_host(host));
    }

    pub(super) fn greet(&mut self, id: SessionId, fresh: bool) {
        if let Some(host) = self.desk.workspace.tab_host(id).cloned() {
            self.dial(id, &host);
            return;
        }
        let line = self.desk.preferences.startup_line().filter(|_| fresh);
        if let (Some(line), Some(session)) = (line, self.desk.sessions.get(id)) {
            session.write(line);
        }
    }

    fn dial(&mut self, id: SessionId, host: &SshHost) {
        if let Some(session) = self.desk.sessions.get(id) {
            session.write(host.command());
        }
        let entry = persist::ssh_entry(host.split().1);
        if let Some(secret) = vault::SSH.load(host) {
            let login = SshLogin::of(host, entry.as_ref());
            self.desk.sessions.expect_secret(id, secret, login);
        }
        let entry = entry.unwrap_or_else(|| HostEntry::of(host));
        let port = host.port.or(entry.port).unwrap_or(SSH_PORT);
        self.desk.sessions.watch_remote(id, entry.address, port);
    }

    pub(in crate::app::running) fn redial(&mut self, now: Instant) -> Option<Instant> {
        let due = self.desk.sessions.redial_due(now);
        for id in &due {
            match self.desk.workspace.tab_host(*id).cloned() {
                Some(host) => self.dial(*id, &host),
                None => self.desk.sessions.forget_remote(*id),
            }
        }
        if !due.is_empty() {
            self.relayout();
            self.window.request_redraw();
        }
        self.desk.sessions.remote_deadline(now)
    }

    pub(in crate::app::running) fn disconnect(&mut self, id: SessionId) {
        self.desk.sessions.forget_remote(id);
        self.relayout();
        self.window.request_redraw();
    }
}
