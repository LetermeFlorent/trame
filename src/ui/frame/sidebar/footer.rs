use crate::geometry::Rect;
use crate::ui::metrics::UiMetrics;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FooterFrame {
    pub rect: Rect,
    pub settings: Rect,
}

impl FooterFrame {
    pub fn above(&self, rect: Rect) -> Rect {
        Rect::from_edges(rect.x, rect.y, rect.right(), self.rect.y.max(rect.y))
    }
}

pub(crate) fn layout_footer(metrics: &UiMetrics, side: Rect) -> FooterFrame {
    let height = metrics.px(UiMetrics::SIDEBAR_HEADER).min(side.height);
    let rect = Rect::new(side.x, side.bottom() - height, side.width, height);
    let size = metrics.px(UiMetrics::BUTTON);
    let rail = metrics.px(UiMetrics::RAIL_WIDTH);
    let settings = Rect::new(
        side.x + (rail - size) / 2.0,
        rect.y + (height - size) / 2.0,
        size,
        size,
    )
    .snapped();
    FooterFrame { rect, settings }
}
