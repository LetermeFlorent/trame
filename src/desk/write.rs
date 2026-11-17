use crate::term::Modes;

use super::{Desk, Effect, Effects};

impl Desk {
    pub fn write_to_active(&mut self, bytes_for: impl FnOnce(Modes) -> Vec<u8>) -> Effects {
        let Some(session) = self.sessions.active(&self.workspace) else {
            return Effects::default();
        };
        let id = session.id();
        if self.sessions.offline(id) {
            return Effects::default();
        }
        let bytes = {
            let mut terminal = session.terminal();
            let bytes = bytes_for(terminal.screen().modes());
            if !bytes.is_empty() {
                let screen = terminal.screen_mut();
                screen.reset_display();
                screen.clear_selection();
            }
            bytes
        };
        if bytes.is_empty() {
            return Effects::default();
        }
        session.write(bytes);
        Effect::Typed(id).into()
    }
}
