use crate::geometry::Rect;
use crate::theme::Swatch;
use crate::ui::metrics::UiMetrics;

const SWATCH_SIZE: f32 = 14.0;
const SWATCH_GAP: f32 = 6.0;
const STRIP_PAD_X: f32 = 8.0;
const STRIP_PAD_Y: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwatchSlot {
    pub rect: Rect,
    pub color: Option<Swatch>,
}

#[derive(Clone, Copy, Debug)]
pub struct SwatchRow {
    pub width: f32,
    pub height: f32,
}

pub fn swatch_row(metrics: &UiMetrics) -> SwatchRow {
    let count = (Swatch::ALL.len() + 1) as f32;
    let size = metrics.px(SWATCH_SIZE);
    let gap = metrics.px(SWATCH_GAP);
    SwatchRow {
        width: count * size + (count - 1.0) * gap + 2.0 * metrics.px(STRIP_PAD_X),
        height: size + 2.0 * metrics.px(STRIP_PAD_Y),
    }
}

impl SwatchRow {
    pub fn slots(&self, metrics: &UiMetrics, strip: Rect) -> Vec<SwatchSlot> {
        let size = metrics.px(SWATCH_SIZE);
        let step = size + metrics.px(SWATCH_GAP);
        let left = strip.x + metrics.px(STRIP_PAD_X);
        let top = strip.y + metrics.px(STRIP_PAD_Y);
        Swatch::ALL
            .iter()
            .map(|swatch| Some(*swatch))
            .chain([None])
            .enumerate()
            .map(|(index, color)| SwatchSlot {
                rect: Rect::new(left + index as f32 * step, top, size, size),
                color,
            })
            .collect()
    }
}
