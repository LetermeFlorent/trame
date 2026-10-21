use std::sync::Arc;

use super::label::fit_label;
use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::{TextLayout, TextSystem};
use crate::ui::metrics::UiMetrics;
use crate::workspace::{Instance, InstanceId, Workspace};

const CAPTION: &str = "INSTANCES";
const CAPTION_MARGIN: f32 = 14.0;
const HEADER_BUTTON_MARGIN: f32 = 8.0;
const ROW_MARGIN: f32 = 6.0;
const ROW_SPACING: f32 = 1.0;
const ICON_MARGIN: f32 = 8.0;
const TAB_INDENT: f32 = 14.0;
const LABEL_GAP: f32 = 8.0;
const CLOSE_MARGIN: f32 = 5.0;

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

struct RowSpec {
    item: SidebarItem,
    label: String,
    count: Option<usize>,
    indent: f32,
    active: bool,
}

pub fn layout_sidebar(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    rect: Rect,
    workspace: &Workspace,
    title: &dyn Fn(SessionId) -> String,
) -> SidebarFrame {
    let (header, list) = rect.split_top(metrics.px(UiMetrics::SIDEBAR_HEADER));
    let button = metrics.px(UiMetrics::BUTTON);
    let new_instance = Rect::new(
        header.right() - metrics.px(HEADER_BUTTON_MARGIN) - button,
        (header.y + (header.height - button) / 2.0).round(),
        button,
        button,
    );
    let step = metrics.px(UiMetrics::SIDEBAR_ROW) + metrics.px(ROW_SPACING);
    let specs: Vec<RowSpec> = workspace
        .instances
        .iter()
        .enumerate()
        .flat_map(|(index, instance)| row_specs(instance, index == workspace.active, title))
        .collect();
    let rows = specs
        .into_iter()
        .enumerate()
        .map(|(index, spec)| layout_row(text, metrics, list, list.y + index as f32 * step, spec))
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

fn row_specs(
    instance: &Instance,
    active: bool,
    title: &dyn Fn(SessionId) -> String,
) -> Vec<RowSpec> {
    let mut specs = vec![RowSpec {
        item: SidebarItem::Instance(instance.id),
        label: instance.name.clone(),
        count: Some(instance.tab_count()),
        indent: 0.0,
        active,
    }];
    if active {
        let current = instance.active_tab();
        specs.extend(instance.tabs().into_iter().map(|tab| RowSpec {
            item: SidebarItem::Tab(tab),
            label: title(tab),
            count: None,
            indent: TAB_INDENT,
            active: Some(tab) == current,
        }));
    }
    specs
}

fn layout_row(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    list: Rect,
    y: f32,
    spec: RowSpec,
) -> SidebarRow {
    let margin = metrics.px(ROW_MARGIN);
    let rect = Rect::new(
        list.x + margin,
        y,
        list.width - 2.0 * margin,
        metrics.px(UiMetrics::SIDEBAR_ROW),
    );
    let icon_size = metrics.px(UiMetrics::ICON);
    let icon = Rect::new(
        rect.x + metrics.px(ICON_MARGIN + spec.indent),
        (rect.y + (rect.height - icon_size) / 2.0).round(),
        icon_size,
        icon_size,
    );
    let close_size = metrics.px(UiMetrics::TAB_CLOSE);
    let close = Rect::new(
        rect.right() - metrics.px(CLOSE_MARGIN) - close_size,
        (rect.y + (rect.height - close_size) / 2.0).round(),
        close_size,
        close_size,
    );
    let label_x = icon.right() + metrics.px(LABEL_GAP);
    let label_width = close.x - metrics.px(LABEL_GAP) - label_x;
    let style = if spec.indent > 0.0 {
        metrics.label()
    } else {
        metrics.strong()
    };
    SidebarRow {
        item: spec.item,
        rect,
        icon,
        close,
        label: fit_label(text, &spec.label, style, label_width),
        label_x,
        count: spec
            .count
            .map(|count| text.layout(&count.to_string(), metrics.caption())),
        active: spec.active,
    }
}
