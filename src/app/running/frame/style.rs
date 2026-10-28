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
        if preferences == self.preferences {
            return;
        }
        let previous = mem::replace(&mut self.preferences, preferences);
        self.apply_system(&previous);
        let current = &self.preferences;
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
        self.saver.touch();
        self.strings = current.language.resolve().strings();
        let layout = |kept: &Preferences| {
            let sizes = (kept.font_size, kept.interface_size);
            (sizes, kept.panel_side, kept.density)
        };
        match layout(&previous) == layout(current) {
            true => self.changed(),
            false => self.apply_scale(),
        }
    }

    fn apply_scale(&mut self) {
        let border = window::frame_border(&self.window);
        self.metrics = self.preferences.ui_metrics(self.window_scale, border);
        let font = self.preferences.font_pixels(self.window_scale);
        self.text.set_terminal_font(font, DEFAULT_LINE_HEIGHT);
        self.changed();
    }

    pub(in crate::app::running) fn set_appearance(&mut self, appearance: Appearance) {
        if appearance == self.theme.appearance {
            return;
        }
        self.theme = Theme::new(appearance);
        self.sessions.set_colors(self.theme.reported_colors());
        self.window.request_redraw();
    }
}
