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
        let cwd = self.start_folder(Placement::Tab);
        if let Err(error) = self.open_in(Placement::Tab, cwd.as_deref(), Some(host.clone())) {
            eprintln!("trame: the SSH tab could not start: {error}");
        }
        self.set_preferences(self.preferences.remember_host(host));
    }

    pub(super) fn greet(&self, id: SessionId) {
        let line = match self.workspace.tab_host(id) {
            Some(host) => Some(host.command()),
            None => self.preferences.startup_line(),
        };
        if let (Some(line), Some(session)) = (line, self.sessions.get(id)) {
            session.write(line);
        }
    }
}
