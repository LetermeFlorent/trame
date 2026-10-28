use trame::preferences::Preferences;
use trame::session::Session;
use trame::term::ReportedColors;

use super::Sessions;

impl Sessions {
    pub(super) fn prepare(&self, session: &Session) {
        let mut terminal = session.terminal();
        let screen = terminal.screen_mut();
        screen.set_history(self.history);
        screen.set_clipboard_writes(self.program_clipboard);
    }

    pub fn follow_preferences(&mut self, preferences: &Preferences) {
        self.set_history(preferences.history_lines);
        self.set_program_clipboard(preferences.program_clipboard);
    }

    pub fn set_colors(&self, colors: ReportedColors) {
        for session in self.map.values() {
            session.terminal().screen_mut().set_reported_colors(colors);
        }
    }

    pub fn set_history(&mut self, lines: usize) {
        self.history = lines;
        for session in self.map.values() {
            session.terminal().screen_mut().set_history(lines);
        }
    }

    pub fn set_program_clipboard(&mut self, allowed: bool) {
        self.program_clipboard = allowed;
        for session in self.map.values() {
            session
                .terminal()
                .screen_mut()
                .set_clipboard_writes(allowed);
        }
    }
}
