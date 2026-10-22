mod rail;
mod row;
mod width;

use std::sync::Arc;

use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::{TextLayout, TextSystem};
use crate::theme::Swatch;
use crate::ui::frame::FrameInput;
use crate::ui::metrics::UiMetrics;
use crate::workspace::InstanceId;
pub use rail::RailFrame;
pub(super) use rail::layout_rail;
use row::{RowSpec, layout_row, row_specs};
pub use width::{dragged_width, shown_width};

const CAPTION: &str = "INSTANCES";
const CAPTION_MARGIN: f32 = 14.0;
const HEADER_BUTTON_MARGIN: f32 = 8.0;
const ROW_SPACING: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarItem {
    Instance(InstanceId),
    Tab(SessionId),
}

#[derive(Clone, Debug)]
pub struct SidebarRow {
    pub item: SidebarItem,
    pub rect: Rect,
    pub icon: Rect,
    pub close: Rect,
    pub label: Arc<TextLayout>,
    pub label_x: f32,
    pub count: Option<Arc<TextLayout>>,
    pub active: bool,
    pub color: Option<Swatch>,
}

#[derive(Clone, Debug)]
pub struct SidebarFrame {
    pub rect: Rect,
    pub header: Rect,
    pub caption: Arc<TextLayout>,
    pub caption_x: f32,
    pub new_instance: Rect,
    pub rows: Vec<SidebarRow>,
}

pub fn layout_sidebar(text: &mut TextSystem, input: &FrameInput, rect: Rect) -> SidebarFrame {
    let metrics = &input.metrics;
    let workspace = input.workspace;
    let (header, list) = rect.split_top(metrics.px(UiMetrics::SIDEBAR_HEADER));
    let button = metrics.px(UiMetrics::BUTTON);
    let new_instance = Rect::new(
        header.right() - metrics.px(HEADER_BUTTON_MARGIN) - button,
        (header.y + (header.height - button) / 2.0).round(),
        button,
        button,
    );
    let height = metrics.px(UiMetrics::SIDEBAR_ROW);
    let step = height + metrics.px(ROW_SPACING);
    let specs: Vec<RowSpec> = workspace
        .instances
        .iter()
        .enumerate()
        .flat_map(|(index, instance)| row_specs(instance, index == workspace.active, input.title))
        .collect();
    let rows = specs
        .into_iter()
        .enumerate()
        .map(|(index, spec)| {
            let slot = Rect::new(list.x, list.y + index as f32 * step, list.width, height);
            layout_row(text, metrics, slot, spec)
        })
        .collect();
    SidebarFrame {
        rect,
        header,
        caption: text.layout(CAPTION, metrics.caption()),
        caption_x: header.x + metrics.px(CAPTION_MARGIN),
        new_instance,
        rows,
    }
}
