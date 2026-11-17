use std::time::Instant;

use crate::desk::{Desk, Effect, Effects};
use crate::session::SessionId;

use super::Placement;

impl Desk {
    pub fn shell_exited(&mut self, exiting: bool, id: SessionId) -> Effects {
        if self.workspace.is_only_tab(id) && !self.restarts.allows(id, Instant::now()) {
            self.show_quick_exit(id)
        } else {
            self.close_tab(exiting, id)
        }
    }

    pub(super) fn replace_last_tab(&mut self) -> Effects {
        let cwd = self.start_folder(Placement::Tab);
        let opened = self.open_in(Placement::Tab, cwd, None);
        self.restarts.opened(opened.value, Instant::now());
        opened.effects
    }

    fn show_quick_exit(&mut self, id: SessionId) -> Effects {
        if let Some(session) = self.sessions.get(id) {
            let notice = format!("\r\n{}\r\n", self.strings.shell_exited_at_once);
            session.terminal().feed(notice.as_bytes());
        }
        Effect::Redraw.into()
    }
}
