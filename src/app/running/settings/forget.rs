use crate::app::running::Running;

impl Running {
    pub(super) fn forget_listed(&mut self, index: usize) {
        let Some(host) = self
            .panel_mut()
            .and_then(|panel| panel.hosts.get(index).cloned())
        else {
            return;
        };
        let effects = self.desk.ask_forget(host);
        self.apply(effects);
    }
}
