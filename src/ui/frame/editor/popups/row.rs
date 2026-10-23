use crate::geometry::Rect;
use crate::theme::Swatch;
use crate::ui::metrics::UiMetrics;

pub(super) const SWATCH_SIZE: f32 = 14.0;
const SWATCH_GAP: f32 = 6.0;
pub(super) const STRIP_PAD_X: f32 = 8.0;
pub(super) const STRIP_PAD_Y: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwatchSlot {
    pub rect: Rect,
    pub color: Option<Swatch>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RowSize {
    pub(super) width: f32,
    pub(super) height: f32,
}

impl RowSize {
    pub(super) fn of(metrics: &UiMetrics, count: usize, slot: f32, gap: f32) -> Self {
        let count = count as f32;
        let size = metrics.px(slot);
        Self {
            width: count * size + (count - 1.0) * metrics.px(gap) + 2.0 * metrics.px(STRIP_PAD_X),
            height: size + 2.0 * metrics.px(STRIP_PAD_Y),
        }
    }
}

pub(super) fn slot_rects(
    metrics: &UiMetrics,
    strip: Rect,
    count: usize,
    slot: f32,
    gap: f32,
) -> impl Iterator<Item = Rect> {
    let size = metrics.px(slot);
    let step = size + metrics.px(gap);
    let left = strip.x + metrics.px(STRIP_PAD_X);
    let top = strip.y + metrics.px(STRIP_PAD_Y);
    (0..count).map(move |index| Rect::new(left + index as f32 * step, top, size, size))
}

pub(super) fn swatch_row(metrics: &UiMetrics) -> RowSize {
    RowSize::of(metrics, Swatch::ALL.len() + 1, SWATCH_SIZE, SWATCH_GAP)
}

pub(super) fn swatch_slots(metrics: &UiMetrics, strip: Rect) -> Vec<SwatchSlot> {
    let colors = Swatch::ALL.iter().map(|swatch| Some(*swatch)).chain([None]);
    let rects = slot_rects(
        metrics,
        strip,
        Swatch::ALL.len() + 1,
        SWATCH_SIZE,
        SWATCH_GAP,
    );
    rects
        .zip(colors)
        .map(|(rect, color)| SwatchSlot { rect, color })
        .collect()
}
