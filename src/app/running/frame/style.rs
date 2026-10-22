use trame::text::DEFAULT_LINE_HEIGHT;
use trame::theme::{Appearance, Theme};
use trame::ui::UiMetrics;
use trame::ui::metrics::{MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE};

use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn rescale(&mut self, scale: f32) {
        self.metrics = UiMetrics::new(scale);
        self.apply_font();
    }

    pub(in crate::app::running) fn set_font_size(&mut self, size: f32) {
        let size = size.clamp(MIN_TERMINAL_FONT_SIZE, MAX_TERMINAL_FONT_SIZE);
        if size != self.font_size {
            self.font_size = size;
            self.apply_font();
        }
    }

    fn apply_font(&mut self) {
        let size = self.font_size * self.metrics.scale;
        self.text.set_terminal_font(size, DEFAULT_LINE_HEIGHT);
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
