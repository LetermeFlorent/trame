use std::time::{Duration, Instant};

use winit::event_loop::ActiveEventLoop;

use trame::log_error;
use trame::session::SessionId;

use super::Placement;
use crate::app::running::Running;

const QUICK_EXIT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub(in crate::app::running) struct Restarts {
    last: Option<(SessionId, Instant)>,
}

impl Restarts {
    pub(super) fn allows(&self, exited: SessionId, now: Instant) -> bool {
        self.last.is_none_or(|(id, opened)| {
            id != exited || now.saturating_duration_since(opened) >= QUICK_EXIT
        })
    }

    pub(super) fn opened(&mut self, id: SessionId, now: Instant) {
        self.last = Some((id, now));
    }
}

impl Running {
    pub(in crate::app::running) fn shell_exited(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: SessionId,
    ) {
        if self.workspace.is_only_tab(id) && !self.restarts.allows(id, Instant::now()) {
            self.show_quick_exit(id);
        } else {
            self.close_tab(event_loop, id);
        }
    }

    pub(super) fn replace_last_tab(&mut self) {
        let cwd = self.start_folder(Placement::Tab);
        match self.open_in(Placement::Tab, cwd.as_deref(), None) {
            Ok(id) => self.restarts.opened(id, Instant::now()),
            Err(error) => log_error!("a fresh tab could not start: {error}"),
        }
    }

    fn show_quick_exit(&mut self, id: SessionId) {
        if let Some(session) = self.sessions.get(id) {
            let notice = format!("\r\n{}\r\n", self.strings.shell_exited_at_once);
            session.terminal().feed(notice.as_bytes());
        }
        self.window.request_redraw();
    }
}
