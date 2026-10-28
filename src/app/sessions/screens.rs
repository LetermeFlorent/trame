use trame::term::ReportedColors;

use super::Sessions;

impl Sessions {
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
}
