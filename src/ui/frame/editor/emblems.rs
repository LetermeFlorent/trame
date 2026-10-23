use super::row::{RowSize, slot_rects};
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

pub(super) fn emblem_row(metrics: &UiMetrics) -> RowSize {
    RowSize::of(metrics, Emblem::ALL.len(), EMBLEM_SLOT, EMBLEM_GAP)
}

pub(super) fn emblem_slots(metrics: &UiMetrics, strip: Rect) -> Vec<EmblemSlot> {
    let size = metrics.px(EMBLEM_SLOT);
    let inset = (size - metrics.px(UiMetrics::ICON)) / 2.0;
    slot_rects(metrics, strip, Emblem::ALL.len(), EMBLEM_SLOT, EMBLEM_GAP)
        .zip(Emblem::ALL)
        .map(|(rect, emblem)| EmblemSlot {
            rect,
            icon: rect.inset(inset),
            emblem,
        })
        .collect()
}
