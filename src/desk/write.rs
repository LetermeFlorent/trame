use crate::term::{Modes, Screen};

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
        let (bytes, redraw) = {
            let mut terminal = session.terminal();
            let bytes = bytes_for(terminal.screen().modes());
            let redraw = !bytes.is_empty() && show_live(terminal.screen_mut());
            (bytes, redraw)
        };
        if bytes.is_empty() {
            return Effects::default();
        }
        session.write(bytes);
        let mut effects = Effects::from(Effect::Typed(id));
        if redraw {
            effects.push(Effect::Redraw);
        }
        effects
    }
}

fn show_live(screen: &mut Screen) -> bool {
    let changed = screen.grid().display_offset() > 0 || screen.selection().is_some();
    screen.reset_display();
    screen.clear_selection();
    changed
}
