use trame::desk::written_by_hand;
use trame::preferences::SshHost;

use crate::app::running::{Closing, Running};

impl Running {
    pub(super) fn forget_listed(&mut self, index: usize) {
        let Some(host) = self.listed_host(index) else {
            return;
        };
        if written_by_hand(&host) {
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
        let effects = self.desk.forget_host(host);
        self.apply(effects);
        self.reload_hosts();
    }
}
