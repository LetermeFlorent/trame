mod footer;
mod row_end;
mod scroll;

use super::{Icon, Painter, Press};
use crate::geometry::Rect;
use crate::theme::{Rgba, Swatch, Theme};
use crate::ui::display::PanelSide;
use crate::ui::frame::{BarPart, RailFrame, RailSlot, SidebarFrame, SidebarItem, SidebarRow};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

pub(super) use footer::draw_footer;
use row_end::draw_row_end;
pub(super) use scroll::draw_bar;

const HOVER_OVER_TINT: f32 = 0.7;
const WORK_HEIGHT: f32 = 2.0;

pub(super) fn draw_panel(painter: &mut Painter, rect: Rect) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    painter.rule(rect, theme.chrome);
    let x = match painter.metrics.panel {
        PanelSide::Left => rect.right() - line,
        PanelSide::Right => rect.x,
    };
    painter.rule(Rect::new(x, rect.y, line, rect.height), theme.border);
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
    if let Some(bar) = sidebar.bar {
        let track = Hit::SidebarBar(BarPart::Track);
        draw_bar(painter, bar, track, Hit::SidebarBar(BarPart::Thumb));
    }
}

fn draw_row(painter: &mut Painter, row: &SidebarRow) {
    let theme = painter.theme;
    let hit = Hit::Row(row.item);
    let glow = painter.glow(hit);
    let (icon, selected) = match row.item {
        SidebarItem::Instance(_) => (Icon::Emblem(row.emblem.unwrap_or_default()), false),
        SidebarItem::Tab(_) => (Icon::Terminal, row.active),
    };
    let solid = row
        .color
        .filter(|_| row.fill)
        .map(|swatch| theme.swatch(swatch));
    let ink = solid.map(Rgba::ink);
    match solid {
        Some(fill) => {
            let radius = painter.metrics.px(UiMetrics::RADIUS);
            painter.shaded_fill(row.rect, hit, fill, radius);
        }
        None => {
            let tint = match row.item {
                SidebarItem::Instance(_) => row.color.map(|swatch| theme.swatch_tint(swatch)),
                SidebarItem::Tab(_) => None,
            };
            draw_backdrop(painter, row.rect, hit, selected, tint);
        }
    }
    let icon_ink = ink.unwrap_or_else(|| icon_color(theme, row.color, row.active));
    painter.icon(icon, row.icon, icon_ink);
    let text_color = ink.unwrap_or_else(|| theme.label_text(row.active, glow));
    if !painter.renaming(|editor| editor.renames_row(row.item)) {
        painter.label(&row.label, row.label_x, row.rect, text_color);
    }
    draw_row_end(painter, row, glow, ink);
    if row.working {
        let radius = painter.metrics.px(UiMetrics::RADIUS);
        let height = painter.metrics.px(WORK_HEIGHT);
        let rect = row.rect;
        let band = Rect::new(rect.x + radius, rect.y, rect.width - radius * 2.0, height);
        painter.work_bar(band, ink.unwrap_or(theme.accent));
    }
}
