use trame::preferences::SshHost;
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
        match self.open(Placement::Tab) {
            Ok(id) => {
                self.workspace.rename_tab(id, &host.label());
                self.workspace.set_tab_host(id, Some(host.clone()));
                self.dial(id);
            }
            Err(error) => eprintln!("trame: the SSH tab could not start: {error}"),
        }
        self.set_preferences(self.preferences.remember_host(host));
    }

    pub(super) fn dial(&self, id: SessionId) {
        let host = self.workspace.tab_host(id);
        if let (Some(host), Some(session)) = (host, self.sessions.get(id)) {
            session.write(host.command());
        }
    }
}
