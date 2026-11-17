use trame::preferences::Preferences;
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
        let effects = self.desk.store_preferences(preferences);
        self.apply(effects);
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
