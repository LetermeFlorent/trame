use crate::preferences::{Preferences, Recall};

use super::Desk;

impl Desk {
    pub(super) fn apply_preferences(&mut self, previous: &Preferences) {
        let current = &self.preferences;
        self.workspace.set_numbering(current.numbering);
        let wanted = current.power_switches();
        if previous.power_switches() != wanted {
            self.power.set_enabled(wanted);
        }
        if previous.recalls(Recall::Text) && !current.recalls(Recall::Text) {
            self.transcripts.clear();
        }
        if previous.history_lines != current.history_lines {
            self.sessions.set_history(current.history_lines);
        }
        if previous.program_clipboard != current.program_clipboard {
            self.sessions
                .set_program_clipboard(current.program_clipboard);
        }
    }
}
