use super::{Icon, Painter};
use crate::geometry::Rect;
use crate::ui::frame::{PaneFrame, TabFrame};
use crate::ui::hit::Hit;

const ACCENT_HEIGHT: f32 = 2.0;

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
    for tab in &strip.tabs {
        draw_tab(painter, pane, tab);
    }
    painter.scene.pop_clip();
    painter.button(strip.new_tab, Hit::NewTab(pane.id), Icon::Plus);
    painter.button(strip.split, Hit::Split(pane.id), Icon::SplitRight);
}

fn draw_tab(painter: &mut Painter, pane: &PaneFrame, tab: &TabFrame) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    let rect = tab.rect;
    let hovered = matches!(
        painter.view.hover,
        Some(Hit::Tab(_, id) | Hit::TabClose(_, id)) if id == tab.id
    );
    let open_bottom = if tab.active { 0.0 } else { line };
    if tab.active {
        painter.rule(rect, theme.tab_active);
    } else if hovered {
        painter.rule(
            Rect::new(rect.x, rect.y, rect.width, rect.height - line),
            theme.hover,
        );
    }
    if tab.active && pane.focused {
        let height = painter.metrics.px(ACCENT_HEIGHT);
        let color = tab
            .color
            .map_or(theme.accent, |swatch| theme.swatch(swatch));
        painter.rule(Rect::new(rect.x, rect.y, rect.width, height), color);
    }
    painter.rule(
        Rect::new(rect.right() - line, rect.y, line, rect.height - open_bottom),
        theme.border,
    );
    let color = match tab.active || hovered {
        true => theme.text,
        false => theme.text_muted,
    };
    painter.label(&tab.label, tab.label_x, rect, color);
    if tab.active || hovered {
        painter.small_button(tab.close, Hit::TabClose(pane.id, tab.id), Icon::Close);
    }
}
