use crate::geometry::{Point, Rect};
use crate::text::TextSystem;
use crate::ui::frame::FrameInput;
use crate::ui::metrics::UiMetrics;

use super::row::{RowSpec, layout_row};
use super::scroll::{BAR_GAP, ListWindow, ScrollBar};
use super::{ROW_SPACING, SidebarFrame, SidebarRow};

const BAR_MARGIN: f32 = 6.0;

impl SidebarFrame {
    pub fn row_at(&self, point: Point) -> Option<usize> {
        if !self.list.contains(point) {
            return None;
        }
        self.rows.iter().position(|row| row.rect.contains(point))
    }
}

pub(super) fn layout_rows(
    text: &mut TextSystem,
    input: &FrameInput,
    list: Rect,
    specs: Vec<RowSpec>,
) -> (Vec<SidebarRow>, Option<ScrollBar>) {
    let metrics = &input.metrics;
    let height = metrics.spaced(UiMetrics::SIDEBAR_ROW);
    let spacing = metrics.px(ROW_SPACING);
    let step = height + spacing;
    let count = specs.len();
    let fits = ((list.height + spacing) / step).floor().max(1.0) as usize;
    let shown = fits.min(count);
    let first = input.workspace.sidebar_first.min(count - shown);
    let window = ListWindow {
        first,
        shown,
        count,
    };
    let bar = (shown < count).then(|| ScrollBar::fit(metrics, bar_area(metrics, list), window));
    let right = bar.map_or(list.right(), |bar| bar.track.x);
    let rows = specs
        .into_iter()
        .enumerate()
        .map(|(index, spec)| {
            let y = list.y + (index as f32 - first as f32) * step;
            let slot = Rect::new(list.x, y, right - list.x, height);
            layout_row(text, metrics, slot, spec)
        })
        .collect();
    (rows, bar)
}

fn bar_area(metrics: &UiMetrics, list: Rect) -> Rect {
    let gap = metrics.px(BAR_GAP);
    let right = list.right() - metrics.px(BAR_MARGIN);
    Rect::from_edges(list.x, list.y + gap, right, list.bottom() - gap)
}
