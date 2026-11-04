use trame::log_error;
use trame::session::SessionId;

use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn start_shown(&mut self) -> bool {
        let shown: Vec<SessionId> = self
            .frame
            .panes
            .iter()
            .filter_map(|pane| pane.active)
            .filter(|id| self.sessions.is_waiting(*id))
            .collect();
        for id in &shown {
            self.start_waiting(*id);
        }
        !shown.is_empty()
    }

    fn start_waiting(&mut self, id: SessionId) {
        let Some(waiting) = self.sessions.take_waiting(id) else {
            return;
        };
        let cwd = self.sessions.cwd(id);
        let text = waiting
            .transcript
            .map(|key| self.transcripts.read(&key).unwrap_or_default());
        if let Err(error) = self.start_shell(id, cwd.as_deref(), text.as_deref()) {
            log_error!("a saved shell could not start: {error}");
        }
    }
}
