use crate::geometry::Rect;
use crate::ui::metrics::UiMetrics;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RailFrame {
    pub rect: Rect,
    pub expand: Rect,
}

pub(crate) fn layout_rail(metrics: &UiMetrics, rect: Rect) -> RailFrame {
    let size = metrics.px(UiMetrics::BUTTON);
    let rail = metrics.px(UiMetrics::RAIL_WIDTH);
    let header = metrics.px(UiMetrics::SIDEBAR_HEADER);
    let expand = Rect::new(
        rect.x + (rail - size) / 2.0,
        rect.y + (header - size) / 2.0,
        size,
        size,
    )
    .snapped();
    RailFrame { rect, expand }
}
