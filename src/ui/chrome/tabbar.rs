use super::{Icon, Painter};
use crate::geometry::Rect;
use crate::ui::frame::{PaneFrame, TabFrame};
use crate::ui::hit::Hit;

const ACCENT_HEIGHT: f32 = 2.0;
const TAB_RADIUS: f32 = 6.0;
const SEPARATOR_INSET: f32 = 7.0;

pub(super) fn draw(painter: &mut Painter, pane: &PaneFrame) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    let strip = &pane.strip;
    painter.rule(strip.rect, theme.tab_bar);
    painter.rule(
        Rect::new(
            strip.rect.x,
            strip.rect.bottom() - line,
            strip.rect.width,
            line,
        ),
        theme.border,
    );
    painter.scene.push_clip(strip.tabs_area);
    for (index, tab) in strip.tabs.iter().enumerate() {
        let before_active = strip.tabs.get(index + 1).is_some_and(|next| next.active);
        draw_tab(painter, pane, tab, before_active);
    }
    painter.scene.pop_clip();
    painter.button(strip.new_tab, Hit::NewTab(pane.id), Icon::Plus);
    painter.button(strip.split, Hit::Split(pane.id), Icon::SplitRight);
}

fn draw_tab(painter: &mut Painter, pane: &PaneFrame, tab: &TabFrame, before_active: bool) {
    let theme = painter.theme;
    let rect = tab.rect;
    let glow = painter.glow(Hit::Tab(pane.id, tab.id));
    painter.scene.push_clip(rect);
    draw_tab_fill(painter, pane, tab, glow);
    if !tab.active && !before_active {
        draw_separator(painter, rect);
    }
    let color = match tab.active {
        true => theme.text,
        false => theme.text_muted.mix(theme.text, glow),
    };
    painter.label(&tab.label, tab.label_x, rect, color);
    let close_opacity = if tab.active { 1.0 } else { glow };
    painter.scene.push_opacity(close_opacity);
    painter.small_button(tab.close, Hit::TabClose(pane.id, tab.id), Icon::Close);
    painter.scene.pop_opacity();
    painter.scene.pop_clip();
}

fn draw_tab_fill(painter: &mut Painter, pane: &PaneFrame, tab: &TabFrame, glow: f32) {
    let theme = painter.theme;
    let radius = painter.metrics.px(TAB_RADIUS);
    let rect = tab.rect;
    if tab.active {
        painter.scene.fill_top(rect, theme.tab_active, radius);
    } else {
        let line = painter.metrics.line();
        let hover = theme.hover.with_alpha(theme.hover.a * glow);
        let above_rule = Rect::new(rect.x, rect.y, rect.width, rect.height - line);
        painter.scene.fill_top(above_rule, hover, radius);
    }
    if tab.active && pane.focused {
        let height = painter.metrics.px(ACCENT_HEIGHT);
        let color = tab
            .color
            .map_or(theme.accent, |swatch| theme.swatch(swatch));
        painter
            .scene
            .push_clip(Rect::new(rect.x, rect.y, rect.width, height));
        painter.scene.fill_top(rect, color, radius);
        painter.scene.pop_clip();
    }
}

fn draw_separator(painter: &mut Painter, rect: Rect) {
    let line = painter.metrics.line();
    let inset = painter.metrics.px(SEPARATOR_INSET);
    let separator = Rect::new(
        rect.right() - line,
        rect.y + inset,
        line,
        rect.height - inset * 2.0,
    );
    let border = painter.theme.border;
    painter.rule(separator, border);
}
