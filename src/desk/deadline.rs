use std::time::Instant;

use crate::workspace::WindowSnapshot;

use super::{Desk, Effects, Outcome};

impl Desk {
    pub fn next_deadline(
        &mut self,
        now: Instant,
        window: impl FnOnce() -> Option<WindowSnapshot>,
    ) -> Outcome<Option<Instant>> {
        let mut effects = Effects::default();
        let save = self.save_deadline(now, window);
        effects.append(save.effects);
        let text = self.transcript_deadline(now);
        let secret = self.sessions.secret_deadline(now);
        let remote = self.redial(now);
        effects.append(remote.effects);
        let value = save
            .value
            .into_iter()
            .chain(text)
            .chain(secret)
            .chain(remote.value)
            .min();
        Outcome { value, effects }
    }
}
