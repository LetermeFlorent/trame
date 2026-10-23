use std::mem;

use trame::preferences::Preferences;
use trame::text::DEFAULT_LINE_HEIGHT;
use trame::theme::{Appearance, Theme};
use trame::ui::UiMetrics;

use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn rescale(&mut self, scale: f32) {
        self.window_scale = scale;
        self.apply_scale();
    }

    pub(in crate::app::running) fn set_preferences(&mut self, preferences: Preferences) {
        let preferences = preferences.sanitized();
        let previous = mem::replace(&mut self.preferences, preferences);
        if previous == preferences {
            return;
        }
        if previous.restore_text && !preferences.restore_text {
            self.transcripts.clear();
        }
        self.saver.touch();
        self.strings = preferences.language.resolve().strings();
        let sizes = |kept: Preferences| (kept.font_size, kept.interface_size);
        match sizes(previous) == sizes(preferences) {
            true => self.changed(),
            false => self.apply_scale(),
        }
    }

    fn apply_scale(&mut self) {
        self.metrics = UiMetrics::new(self.preferences.ui_scale(self.window_scale));
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
