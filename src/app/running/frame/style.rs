use std::mem;

use trame::preferences::{Preferences, Recall};
use trame::text::DEFAULT_LINE_HEIGHT;
use trame::theme::{Appearance, Theme};

use crate::app::running::Running;
use crate::app::window;

impl Running {
    pub(in crate::app::running) fn rescale(&mut self, scale: f32) {
        self.window_scale = scale;
        self.apply_scale();
    }

    pub(in crate::app::running) fn set_preferences(&mut self, preferences: Preferences) {
        let preferences = preferences.sanitized();
        if preferences == self.desk.preferences {
            return;
        }
        let previous = mem::replace(&mut self.desk.preferences, preferences);
        self.apply_system(&previous);
        let current = &self.desk.preferences;
        if previous.recalls(Recall::Text) && !current.recalls(Recall::Text) {
            self.desk.transcripts.clear();
        }
        if previous.history_lines != current.history_lines {
            self.desk.sessions.set_history(current.history_lines);
        }
        if previous.program_clipboard != current.program_clipboard {
            self.desk
                .sessions
                .set_program_clipboard(current.program_clipboard);
        }
        self.desk.saver.touch();
        self.desk.strings = current.language.resolve().strings();
        let layout = |kept: &Preferences| {
            let sizes = (kept.font_size, kept.interface_size);
            (sizes, kept.panel_side, kept.density)
        };
        match layout(&previous) == layout(current) {
            true => self.changed(),
            false => self.apply_scale(),
        }
    }

    pub(in crate::app::running) fn apply_scale(&mut self) {
        let border = window::frame_border(&self.window);
        self.metrics = self.desk.preferences.ui_metrics(self.window_scale, border);
        let font = self.desk.preferences.font_pixels(self.window_scale);
        self.text.set_terminal_font(font, DEFAULT_LINE_HEIGHT);
        self.changed();
    }

    pub(in crate::app::running) fn set_appearance(&mut self, appearance: Appearance) {
        if appearance == self.desk.theme.appearance {
            return;
        }
        self.desk.theme = Theme::new(appearance);
        self.desk
            .sessions
            .set_colors(self.desk.theme.reported_colors());
        self.window.request_redraw();
    }
}
