use std::sync::Arc;

use crate::geometry::Rect;
use crate::text::{TextLayout, TextSystem};
use crate::ui::metrics::UiMetrics;

const LABEL_GAP: f32 = 2.0;

#[derive(Clone, Debug)]
pub struct FooterFrame {
    pub rect: Rect,
    pub settings: Rect,
    pub icon: Rect,
    pub label: Arc<TextLayout>,
    pub label_x: f32,
}

impl FooterFrame {
    pub fn above(&self, rect: Rect) -> Rect {
        Rect::from_edges(rect.x, rect.y, rect.right(), self.rect.y.max(rect.y))
    }
}

pub(crate) fn layout_footer(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    side: Rect,
    label: &str,
) -> FooterFrame {
    let height = metrics.px(UiMetrics::SIDEBAR_HEADER).min(side.height);
    let rect = Rect::new(side.x, side.bottom() - height, side.width, height);
    let size = metrics.px(UiMetrics::BUTTON);
    let inset = (metrics.px(UiMetrics::RAIL_WIDTH) - size) / 2.0;
    let y = rect.y + (height - size) / 2.0;
    let width = (side.width - 2.0 * inset).max(size);
    let settings = Rect::new(side.x + inset, y, width, size).snapped();
    let icon = Rect::new(settings.x, settings.y, size, size);
    FooterFrame {
        rect,
        settings,
        icon,
        label: text.layout(label, metrics.label()),
        label_x: icon.right() + metrics.px(LABEL_GAP),
    }
}
