use super::{Icon, Painter, Press};
use crate::geometry::Rect;
use crate::theme::{Rgba, Swatch, Theme};
use crate::ui::frame::{RailFrame, RailSlot, SidebarFrame, SidebarItem, SidebarRow};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

const HOVER_OVER_TINT: f32 = 0.7;

pub(super) fn draw_panel(painter: &mut Painter, rect: Rect) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    painter.rule(rect, theme.chrome);
    painter.rule(
        Rect::new(rect.right() - line, rect.y, line, rect.height),
        theme.border,
    );
}

pub(super) fn draw_rail(painter: &mut Painter, rail: &RailFrame) {
    painter.scene.push_clip(rail.rect);
    for slot in &rail.slots {
        draw_slot(painter, slot);
    }
    painter.scene.pop_clip();
}

fn draw_slot(painter: &mut Painter, slot: &RailSlot) {
    let theme = painter.theme;
    let tint = slot.color.map(|swatch| theme.swatch_tint(swatch));
    draw_backdrop(
        painter,
        slot.rect,
        Hit::RailInstance(slot.id),
        slot.active,
        tint,
    );
    let color = icon_color(theme, slot.color, slot.active);
    painter.icon(Icon::Emblem(slot.emblem), slot.icon, color);
}

fn draw_backdrop(painter: &mut Painter, rect: Rect, hit: Hit, selected: bool, tint: Option<Rgba>) {
    let theme = painter.theme;
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    if let Some(color) = tint {
        painter.scene.fill(rect, color, radius);
    }
    let (fill, strength) = match painter.state(hit) {
        Press::Pressed => (theme.pressed, 1.0),
        _ if selected => (theme.hover, 1.0),
        _ => (theme.hover, painter.glow(hit)),
    };
    let alpha = tint.map_or(fill.a, |_| HOVER_OVER_TINT);
    painter
        .scene
        .fill(rect, fill.with_alpha(alpha * strength), radius);
}

fn icon_color(theme: &Theme, color: Option<Swatch>, active: bool) -> Rgba {
    match (color, active) {
        (Some(swatch), _) => theme.swatch(swatch),
        (None, true) => theme.accent,
        (None, false) => theme.text_faint,
    }
}

pub(super) fn draw(painter: &mut Painter, sidebar: &SidebarFrame) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    let rect = sidebar.rect;
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
    let hit = Hit::Row(row.item);
    let glow = painter.glow(hit);
    let (icon, selected) = match row.item {
        SidebarItem::Instance(_) => (Icon::Emblem(row.emblem.unwrap_or_default()), false),
        SidebarItem::Tab(_) => (Icon::Terminal, row.active),
    };
    let tint = match row.item {
        SidebarItem::Instance(_) => row.color.map(|swatch| theme.swatch_tint(swatch)),
        SidebarItem::Tab(_) => None,
    };
    draw_backdrop(painter, row.rect, hit, selected, tint);
    painter.icon(icon, row.icon, icon_color(theme, row.color, row.active));
    let text_color = match row.active {
        true => theme.text,
        false => theme.text_muted.mix(theme.text, glow),
    };
    painter.label(&row.label, row.label_x, row.rect, text_color);
    draw_row_end(painter, row, glow);
}

fn draw_row_end(painter: &mut Painter, row: &SidebarRow, glow: f32) {
    painter.scene.push_opacity(glow);
    painter.small_button(row.close, Hit::RowClose(row.item), Icon::Close);
    painter.scene.pop_opacity();
    if let Some(count) = &row.count {
        let x = row.close.right() - count.width;
        painter.scene.push_opacity(1.0 - glow);
        painter.label(count, x, row.rect, painter.theme.text_faint);
        painter.scene.pop_opacity();
    }
}
