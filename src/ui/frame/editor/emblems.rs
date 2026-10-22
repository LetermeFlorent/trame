use super::row::{STRIP_PAD_X, STRIP_PAD_Y};
use crate::geometry::Rect;
use crate::theme::Emblem;
use crate::ui::metrics::UiMetrics;

const EMBLEM_SLOT: f32 = 18.0;
const EMBLEM_GAP: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmblemSlot {
    pub rect: Rect,
    pub icon: Rect,
    pub emblem: Emblem,
}

pub(super) fn emblem_row_width(metrics: &UiMetrics) -> f32 {
    let count = Emblem::ALL.len() as f32;
    count * metrics.px(EMBLEM_SLOT)
        + (count - 1.0) * metrics.px(EMBLEM_GAP)
        + 2.0 * metrics.px(STRIP_PAD_X)
}

pub(super) fn emblem_row_height(metrics: &UiMetrics) -> f32 {
    metrics.px(EMBLEM_SLOT) + metrics.px(STRIP_PAD_Y)
}

pub(super) fn emblem_slots(metrics: &UiMetrics, strip: Rect) -> Vec<EmblemSlot> {
    let size = metrics.px(EMBLEM_SLOT);
    let icon = metrics.px(UiMetrics::ICON);
    let step = size + metrics.px(EMBLEM_GAP);
    let left = strip.x + metrics.px(STRIP_PAD_X);
    let top = strip.bottom() - metrics.px(STRIP_PAD_Y) - size;
    Emblem::ALL
        .into_iter()
        .enumerate()
        .map(|(index, emblem)| {
            let rect = Rect::new(left + index as f32 * step, top, size, size);
            EmblemSlot {
                rect,
                icon: rect.inset_by((size - icon) / 2.0, (size - icon) / 2.0),
                emblem,
            }
        })
        .collect()
}
