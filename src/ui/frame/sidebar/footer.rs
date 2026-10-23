use std::sync::Arc;

use crate::geometry::Rect;
use crate::text::{TextLayout, TextSystem};
use crate::ui::frame::FrameInput;
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

pub(crate) fn layout_footer(text: &mut TextSystem, input: &FrameInput, side: Rect) -> FooterFrame {
    let metrics = &input.metrics;
    let line = metrics.line();
    let size = metrics.px(UiMetrics::BUTTON);
    let gap = ((metrics.px(UiMetrics::SIDEBAR_HEADER) - line - size) / 2.0)
        .floor()
        .max(0.0);
    let shown = side.height - metrics.hidden_bottom(input.maximized);
    let height = (line + size + 2.0 * gap).min(shown);
    let rect = Rect::new(side.x, side.y + shown - height, side.width, height);
    let inset = (metrics.px(UiMetrics::RAIL_WIDTH) - size) / 2.0;
    let y = rect.bottom() - gap - size;
    let width = (side.width - 2.0 * inset).max(size);
    let settings = Rect::new(side.x + inset, y, width, size).snapped();
    let icon = Rect::new(settings.x, settings.y, size, size);
    FooterFrame {
        rect,
        settings,
        icon,
        label: text.layout(input.strings.settings, metrics.label()),
        label_x: icon.right() + metrics.px(LABEL_GAP),
    }
}
