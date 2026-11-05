use crate::geometry::Rect;
use crate::ui::display::{PanelSide, SidebarMode};
use crate::ui::frame::FrameInput;
use crate::ui::metrics::UiMetrics;

const MAX_WINDOW_SHARE: f32 = 0.5;

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
    pub(in crate::ui::frame) reveal: f32,
}

impl Columns {
    pub(in crate::ui::frame) fn split(input: &FrameInput, rest: Rect, extent: f32) -> Self {
        let metrics = input.metrics;
        let rail_extent = SidebarMode::Rail.extent();
        let reveal = (extent - rail_extent).clamp(0.0, 1.0);
        let rail_width = metrics.px(UiMetrics::RAIL_WIDTH);
        let full_width = metrics
            .px(shown_width(input.workspace.sidebar_width))
            .min(rest.width * MAX_WINDOW_SHARE)
            .round();
        let rail_visible = rail_width * extent.clamp(0.0, rail_extent);
        let visible_width = (rail_visible + (full_width - rail_visible) * reveal).round();
        let shown = match input.workspace.sidebar {
            SidebarMode::Open => full_width,
            SidebarMode::Rail => rail_width,
            SidebarMode::Hidden => 0.0,
        };
        let (side, content) = match metrics.panel {
            PanelSide::Left => (rest.split_left(visible_width).0, rest.split_left(shown).1),
            PanelSide::Right => (
                rest.split_left(rest.width - visible_width).1,
                rest.split_left(rest.width - shown).0,
            ),
        };
        let x = match metrics.panel {
            PanelSide::Left => side.right() - full_width,
            PanelSide::Right => side.x,
        };
        Self {
            side,
            sidebar: Rect::new(x, side.y, full_width, side.height),
            content,
            reveal,
        }
    }

    pub(in crate::ui::frame) fn has_rail(&self) -> bool {
        self.reveal < 1.0 && self.side.width > 0.0
    }
}
