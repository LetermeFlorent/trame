use std::time::{Duration, Instant};

use crate::session::SessionId;

const QUICK_EXIT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct Restarts {
    last: Option<(SessionId, Instant)>,
}

impl Restarts {
    pub fn allows(&self, exited: SessionId, now: Instant) -> bool {
        self.last.is_none_or(|(id, opened)| {
            id != exited || now.saturating_duration_since(opened) >= QUICK_EXIT
        })
    }

    pub fn opened(&mut self, id: SessionId, now: Instant) {
        self.last = Some((id, now));
    }
}
