use super::{
    INTERFACE_SIZE, INTERFACE_SIZE_STEP, MAX_INTERFACE_SIZE, MIN_INTERFACE_SIZE, Preferences,
};
use crate::ui::metrics::{
    MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE, TERMINAL_FONT_SIZE, UiMetrics,
};

impl Preferences {
    pub fn ui_metrics(&self, window_scale: f32, frame_border: f32) -> UiMetrics {
        UiMetrics::new(self.ui_scale(window_scale))
            .with_frame_border(frame_border)
            .with_display(self.panel_side, self.density)
    }
}

pub(super) fn interface_size() -> u16 {
    INTERFACE_SIZE
}

pub(super) fn clamp_font_size(size: f32) -> f32 {
    match size.is_finite() {
        true => size.clamp(MIN_TERMINAL_FONT_SIZE, MAX_TERMINAL_FONT_SIZE),
        false => TERMINAL_FONT_SIZE,
    }
}

pub(super) fn clamp_interface_size(size: u16) -> u16 {
    let step = INTERFACE_SIZE_STEP;
    let rounded = size.saturating_add(step / 2) / step * step;
    rounded.clamp(MIN_INTERFACE_SIZE, MAX_INTERFACE_SIZE)
}
