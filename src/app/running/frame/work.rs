use std::time::{Duration, Instant};

use super::Running;

const WORK_FRAME: Duration = Duration::from_millis(33);

impl Running {
    pub(in crate::app::running) fn work_wake(&mut self, now: Instant) -> Option<Instant> {
        if !self.frame.shows_work() || self.visibility.is_hidden(&self.window) {
            return None;
        }
        let next = self.motion.now() + WORK_FRAME;
        if next > now {
            return Some(next);
        }
        self.window.request_redraw();
        Some(now + WORK_FRAME)
    }
}
