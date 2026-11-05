use super::{SidebarItem, SidebarRow};
use crate::geometry::Rect;
use crate::text::TextSystem;
use crate::ui::frame::{FrameInput, fit_label};
use crate::ui::metrics::UiMetrics;
use crate::workspace::Instance;

const ROW_MARGIN: f32 = 6.0;
const ICON_MARGIN: f32 = 8.0;
const TAB_INDENT: f32 = 14.0;
const LABEL_GAP: f32 = 8.0;

pub(super) struct RowSpec {
    item: SidebarItem,
    label: String,
    count: Option<usize>,
    indent: f32,
    active: bool,
    folded: Option<bool>,
}

pub(super) fn row_specs(instance: &Instance, active: bool, input: &FrameInput) -> Vec<RowSpec> {
    let mut specs = vec![RowSpec {
        item: SidebarItem::Instance(instance.id),
        label: instance
            .shown_name(input.strings.instance_name)
            .into_owned(),
        count: Some(instance.tab_count()),
        indent: 0.0,
        active,
        folded: Some(instance.folded),
    }];
    if !instance.folded {
        let current = instance.active_tab();
        specs.extend(instance.tabs().into_iter().map(|tab| RowSpec {
            item: SidebarItem::Tab(tab),
            label: (input.title)(tab),
            count: None,
            indent: TAB_INDENT,
            active: Some(tab) == current,
            folded: None,
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
    let rect = slot.inset_by(margin, 0.0);
    let icon_size = metrics.px(UiMetrics::ICON);
    let icon = rect.square_at(rect.x + metrics.px(ICON_MARGIN + spec.indent), icon_size);
    let close_size = metrics.px(UiMetrics::TAB_CLOSE);
    let close_x = rect.right() - metrics.px(UiMetrics::TAB_CLOSE_MARGIN) - close_size;
    let close = rect.square_at(close_x, close_size);
    let gap = metrics.px(LABEL_GAP);
    let fold = spec
        .folded
        .map(|_| rect.square_at(close.x - gap - close_size, close_size));
    let label_x = icon.right() + gap;
    let label_width = fold.unwrap_or(close).x - gap - label_x;
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
        fold,
        folded: spec.folded.unwrap_or(false),
        label: fit_label(text, &spec.label, style, label_width),
        label_x,
        count: spec
            .count
            .map(|count| text.layout(&count.to_string(), metrics.caption())),
        active: spec.active,
        color: None,
        fill: false,
        emblem: None,
        working: false,
    }
}
