use trame::desk::{Closing, written_by_hand};

use crate::app::running::Running;

impl Running {
    pub(super) fn forget_listed(&mut self, index: usize) {
        let Some(host) = self
            .panel_mut()
            .and_then(|panel| panel.hosts.get(index).cloned())
        else {
            return;
        };
        if host.alias().is_some() && written_by_hand(&host) {
            self.desk.closing = Some(Closing::Forget(host));
            return;
        }
        let effects = self.desk.forget_host(&host);
        self.apply(effects);
        self.reload_hosts();
    }
}
