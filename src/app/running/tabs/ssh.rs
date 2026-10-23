use trame::preferences::{SshHost, vault};
use trame::session::SessionId;
use trame::ui::frame::EditTarget;

use super::Placement;
use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn connect_saved(&mut self, target: EditTarget, index: usize) {
        if let Some(host) = self.preferences.ssh_hosts.get(index).cloned() {
            self.connect(target, host);
        }
    }

    pub(in crate::app::running) fn connect(&mut self, target: EditTarget, host: SshHost) {
        match target {
            EditTarget::Tab(id) => self.workspace.activate_tab(id),
            EditTarget::Instance(id) => self.activate_instance(id),
        }
        let cwd = self.start_folder(Placement::Tab);
        if let Err(error) = self.open_in(Placement::Tab, cwd.as_deref(), Some(host.clone())) {
            eprintln!("trame: the SSH tab could not start: {error}");
        }
        let next = self.preferences.remember_host(host);
        let dropped = self.preferences.ssh_hosts.iter();
        for host in dropped.filter(|host| !next.ssh_hosts.contains(host)) {
            vault::SSH.forget(host);
        }
        self.set_preferences(next);
    }

    pub(super) fn greet(&mut self, id: SessionId) {
        let host = self.workspace.tab_host(id).cloned();
        let line = match &host {
            Some(host) => Some(host.command()),
            None => self.preferences.startup_line(),
        };
        if let (Some(line), Some(session)) = (line, self.sessions.get(id)) {
            session.write(line);
        }
        if let Some(secret) = host.as_ref().and_then(|host| vault::SSH.load(host)) {
            self.sessions.expect_secret(id, secret);
        }
    }
}
