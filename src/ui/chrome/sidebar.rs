use super::{Icon, Painter, Press};
use crate::geometry::Rect;
use crate::ui::frame::{SidebarFrame, SidebarItem, SidebarRow};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

pub(super) fn draw(painter: &mut Painter, sidebar: &SidebarFrame) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    let rect = sidebar.rect;
    painter.rule(rect, theme.chrome);
    painter.rule(
        Rect::new(rect.right() - line, rect.y, line, rect.height),
        theme.border,
    );
    painter.label(
        &sidebar.caption,
        sidebar.caption_x,
        sidebar.header,
        theme.text_faint,
    );
    painter.button(sidebar.new_instance, Hit::NewInstance, Icon::Plus);
    let list = Rect::from_edges(
        rect.x,
        sidebar.header.bottom(),
        rect.right() - line,
        rect.bottom(),
    );
    painter.scene.push_clip(list);
    for row in &sidebar.rows {
        draw_row(painter, row);
    }
    painter.scene.pop_clip();
}

fn draw_row(painter: &mut Painter, row: &SidebarRow) {
    let theme = painter.theme;
    let hovered = matches!(
        painter.view.hover,
        Some(Hit::Row(item) | Hit::RowClose(item)) if item == row.item
    );
    let pressed = painter.state(Hit::Row(row.item)) == Press::Pressed;
    let (icon, selected) = match row.item {
        SidebarItem::Instance(_) => (Icon::Instance, false),
        SidebarItem::Tab(_) => (Icon::Terminal, row.active),
    };
    let background = match () {
        _ if pressed => Some(theme.pressed),
        _ if hovered || selected => Some(theme.hover),
        _ => None,
    };
    if let Some(color) = background {
        let radius = painter.metrics.px(UiMetrics::RADIUS);
        painter.scene.fill(row.rect, color, radius);
    }
    let icon_color = match row.active {
        true => theme.accent,
        false => theme.text_faint,
    };
    painter.icon(icon, row.icon, icon_color);
    let text_color = match row.active || hovered {
        true => theme.text,
        false => theme.text_muted,
    };
    painter.label(&row.label, row.label_x, row.rect, text_color);
    if hovered {
        painter.small_button(row.close, Hit::RowClose(row.item), Icon::Close);
    } else if let Some(count) = &row.count {
        let x = row.close.right() - count.width;
        painter.label(count, x, row.rect, theme.text_faint);
    }
}
