use super::{SidebarItem, SidebarRow};
use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::TextSystem;
use crate::ui::frame::fit_label;
use crate::ui::metrics::UiMetrics;
use crate::workspace::Instance;

const ROW_MARGIN: f32 = 6.0;
const ICON_MARGIN: f32 = 8.0;
const TAB_INDENT: f32 = 14.0;
const LABEL_GAP: f32 = 8.0;
const CLOSE_MARGIN: f32 = 5.0;

pub(super) struct RowSpec {
    item: SidebarItem,
    label: String,
    count: Option<usize>,
    indent: f32,
    active: bool,
}

pub(super) fn row_specs(
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

pub(super) fn layout_row(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    slot: Rect,
    spec: RowSpec,
) -> SidebarRow {
    let margin = metrics.px(ROW_MARGIN);
    let rect = Rect::new(
        slot.x + margin,
        slot.y,
        slot.width - 2.0 * margin,
        slot.height,
    );
    let icon_size = metrics.px(UiMetrics::ICON);
    let icon = square_in(
        rect,
        rect.x + metrics.px(ICON_MARGIN + spec.indent),
        icon_size,
    );
    let close_size = metrics.px(UiMetrics::TAB_CLOSE);
    let close_x = rect.right() - metrics.px(CLOSE_MARGIN) - close_size;
    let close = square_in(rect, close_x, close_size);
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
        color: None,
    }
}

fn square_in(row: Rect, x: f32, size: f32) -> Rect {
    Rect::new(x, (row.y + (row.height - size) / 2.0).round(), size, size)
}
