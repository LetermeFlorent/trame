use crate::ui::metrics::UiMetrics;

use super::super::column::{CAPTION_HEIGHT, LINE_HEIGHT, SECTION_GAP};

const CAPTIONS: f32 = 4.0;
const LINES: f32 = 13.0;
const SYSTEM_LINES: f32 = 2.0;

pub(in super::super) fn general_height(metrics: &UiMetrics) -> f32 {
    let system = f32::from(u8::from(cfg!(windows)));
    let captions = CAPTIONS + system;
    let lines = LINES + system * SYSTEM_LINES;
    captions * metrics.px(CAPTION_HEIGHT)
        + lines * metrics.px(LINE_HEIGHT)
        + (captions - 1.0) * metrics.px(SECTION_GAP)
}
