use crate::geometry::Rect;
use crate::ui::frame::FrameInput;
use crate::ui::metrics::UiMetrics;

pub fn shown_width(stored: Option<f32>) -> f32 {
    stored
        .unwrap_or(UiMetrics::SIDEBAR_WIDTH)
        .clamp(UiMetrics::SIDEBAR_MIN_WIDTH, UiMetrics::SIDEBAR_MAX_WIDTH)
}

pub fn dragged_width(pointer: f32) -> Option<f32> {
    (pointer >= UiMetrics::SIDEBAR_COLLAPSE_WIDTH).then(|| shown_width(Some(pointer)))
}

pub(in crate::ui::frame) struct Columns {
    pub(in crate::ui::frame) side: Rect,
    pub(in crate::ui::frame) sidebar: Rect,
    pub(in crate::ui::frame) content: Rect,
}

impl Columns {
    pub(in crate::ui::frame) fn split(input: &FrameInput, rest: Rect, reveal: f32) -> Self {
        let metrics = input.metrics;
        let rail_width = metrics.px(UiMetrics::RAIL_WIDTH);
        let full_width = metrics
            .px(shown_width(input.workspace.sidebar_width))
            .round();
        let visible_width = (rail_width + (full_width - rail_width) * reveal).round();
        let (side, _) = rest.split_left(visible_width);
        let shown = if input.workspace.sidebar_open {
            full_width
        } else {
            rail_width
        };
        let (_, content) = rest.split_left(shown);
        Self {
            side,
            sidebar: Rect::new(side.right() - full_width, side.y, full_width, side.height),
            content,
        }
    }
}
