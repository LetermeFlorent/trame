use crate::ui::metrics::UiMetrics;

pub fn shown_width(stored: Option<f32>) -> f32 {
    stored
        .unwrap_or(UiMetrics::SIDEBAR_WIDTH)
        .clamp(UiMetrics::SIDEBAR_MIN_WIDTH, UiMetrics::SIDEBAR_MAX_WIDTH)
}

pub fn dragged_width(pointer: f32) -> Option<f32> {
    (pointer >= UiMetrics::SIDEBAR_COLLAPSE_WIDTH).then(|| shown_width(Some(pointer)))
}
