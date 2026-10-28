use std::time::Instant;

use trame::preferences::SecretAnswer;
use trame::session::SessionId;

use super::Sessions;

impl Sessions {
    pub fn expect_secret(&mut self, id: SessionId, secret: String) {
        self.answers
            .insert(id, SecretAnswer::new(secret, Instant::now()));
    }

    pub(super) fn answer_prompt(&mut self, id: SessionId) {
        let (Some(answer), Some(session)) = (self.answers.get_mut(&id), self.map.get(&id)) else {
            return;
        };
        let now = Instant::now();
        let line = session.terminal().screen().cursor_line();
        if let Some(reply) = answer.reply(&line, now) {
            session.write(reply);
        }
        if answer.is_over(now) {
            self.answers.remove(&id);
        }
    }

    pub fn secret_deadline(&mut self, now: Instant) -> Option<Instant> {
        self.answers.retain(|_, answer| !answer.is_over(now));
        self.answers.values().map(SecretAnswer::deadline).min()
    }
}
