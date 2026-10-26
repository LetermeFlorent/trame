use super::{
    FONT_SIZE_STEP, INTERFACE_SIZE, INTERFACE_SIZE_STEP, MAX_INTERFACE_SIZE, MIN_INTERFACE_SIZE,
    Preferences,
};
use crate::ui::metrics::{
    MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE, TERMINAL_FONT_SIZE, UiMetrics,
};

const PERCENT: f32 = 100.0;

impl Preferences {
    pub fn with_font_size(&self, font_size: f32) -> Self {
        self.with(|next| next.font_size = clamp_font_size(font_size))
    }

    pub fn step_font_size(&self, steps: i32) -> Self {
        self.with_font_size(self.font_size + steps as f32 * FONT_SIZE_STEP)
    }

    pub fn step_interface_size(&self, steps: i32) -> Self {
        let wanted = i32::from(self.interface_size) + steps * i32::from(INTERFACE_SIZE_STEP);
        let size = wanted.clamp(0, i32::from(u16::MAX)) as u16;
        self.with(|next| next.interface_size = clamp_interface_size(size))
    }

    pub fn ui_scale(&self, window_scale: f32) -> f32 {
        window_scale * f32::from(self.interface_size) / PERCENT
    }

    pub fn font_pixels(&self, window_scale: f32) -> f32 {
        self.font_size * window_scale
    }

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
