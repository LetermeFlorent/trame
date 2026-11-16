use std::time::Instant;

use trame::log_error;
use trame::session::SessionId;

use super::Placement;
use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn shell_exited(&mut self, exiting: bool, id: SessionId) {
        if self.desk.workspace.is_only_tab(id) && !self.desk.restarts.allows(id, Instant::now()) {
            self.show_quick_exit(id);
        } else {
            self.close_tab(exiting, id);
        }
    }

    pub(super) fn replace_last_tab(&mut self) {
        let cwd = self.start_folder(Placement::Tab);
        match self.open_in(Placement::Tab, cwd.as_deref(), None) {
            Ok(id) => self.desk.restarts.opened(id, Instant::now()),
            Err(error) => log_error!("a fresh tab could not start: {error}"),
        }
    }

    fn show_quick_exit(&mut self, id: SessionId) {
        if let Some(session) = self.desk.sessions.get(id) {
            let notice = format!("\r\n{}\r\n", self.desk.strings.shell_exited_at_once);
            session.terminal().feed(notice.as_bytes());
        }
        self.window.request_redraw();
    }
}
