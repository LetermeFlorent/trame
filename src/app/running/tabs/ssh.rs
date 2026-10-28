use trame::preferences::{SshHost, SshLogin, menu_hosts, vault};
use trame::session::SessionId;
use trame::ui::frame::EditTarget;

use super::Placement;
use crate::app::persist;
use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn known_hosts(&self) -> Vec<SshHost> {
        let aliases = persist::ssh_config().map_or(Ok(Vec::new()), |config| config.aliases());
        let aliases = aliases.unwrap_or_else(|error| {
            eprintln!("trame: the SSH config could not be read: {error}");
            Vec::new()
        });
        menu_hosts(&self.preferences.ssh_hosts, &aliases)
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

    pub(super) fn greet(&mut self, id: SessionId, fresh: bool) {
        let host = self.workspace.tab_host(id).cloned();
        let line = match &host {
            Some(host) => Some(host.command()),
            None => self.preferences.startup_line().filter(|_| fresh),
        };
        if let (Some(line), Some(session)) = (line, self.sessions.get(id)) {
            session.write(line);
        }
        if let Some(host) = &host
            && let Some(secret) = vault::SSH.load(host)
        {
            let entry = persist::ssh_entry(host.split().1);
            let login = SshLogin::of(host, entry.as_ref());
            self.sessions.expect_secret(id, secret, login);
        }
    }
}
