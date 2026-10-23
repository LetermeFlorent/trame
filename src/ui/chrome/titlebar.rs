use super::{Icon, Painter, Press, centered};
use crate::geometry::Rect;
use crate::theme::Rgba;
use crate::ui::frame::{TitleBarFrame, WindowControls};
use crate::ui::hit::Hit;
use crate::ui::metrics::Platform;

const SQUARE_GLYPH: f32 = 10.0;
const SQUARE_CROSS: f32 = 14.0;
const ROUND_GLYPH: f32 = 8.0;
const ROUND_CROSS: f32 = 12.0;
const CLOSE_RED: Rgba = Rgba::hex(0xc42b1c);
const CLOSE_PRESSED_SHADE: f32 = 0.15;
const ROUND_IDLE_ALPHA: f32 = 0.7;

pub(super) fn draw(painter: &mut Painter, bar: &TitleBarFrame, sidebar_open: bool) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    painter.rule(bar.rect, theme.chrome);
    painter.rule(
        Rect::new(bar.rect.x, bar.rect.bottom() - line, bar.rect.width, line),
        theme.border,
    );
    painter.button(
        bar.toggle,
        Hit::SidebarToggle,
        Icon::Sidebar { open: sidebar_open },
    );
    let title_color = match painter.view.window_focused {
        true => theme.text,
        false => theme.text_muted,
    };
    if let Some((layout, x)) = &bar.title {
        painter.label(layout, *x, bar.rect, title_color);
    }
    if let Some((layout, x)) = &bar.subtitle {
        painter.label(layout, *x, bar.rect, theme.text_faint);
    }
    if let Some((layout, x)) = &bar.brand {
        painter.label(layout, *x, bar.rect, theme.text_muted);
    }
    if let Some(controls) = &bar.controls {
        draw_controls(painter, controls, bar.maximized);
    }
}

fn draw_controls(painter: &mut Painter, controls: &WindowControls, maximized: bool) {
    let toggle = match maximized {
        true => Icon::Restore,
        false => Icon::Maximize,
    };
    let buttons = [
        (controls.minimize, Hit::Minimize, Icon::Minimize),
        (controls.maximize, Hit::Maximize, toggle),
        (controls.close, Hit::Close, Icon::Close),
    ];
    for (rect, hit, icon) in buttons {
        draw_control(painter, rect, hit, icon);
    }
}

fn draw_control(painter: &mut Painter, rect: Rect, hit: Hit, icon: Icon) {
    let theme = painter.theme;
    let round = painter.metrics.platform != Platform::Windows;
    let state = painter.state(hit);
    let closing = hit == Hit::Close && state != Press::Idle;
    let background = match state {
        _ if closing && state == Press::Pressed => {
            Some(CLOSE_RED.mix(Rgba::hex(0x000000), CLOSE_PRESSED_SHADE))
        }
        _ if closing => Some(CLOSE_RED),
        Press::Hover => Some(theme.hover),
        Press::Pressed => Some(theme.pressed),
        Press::Idle if round => Some(theme.hover.with_alpha(ROUND_IDLE_ALPHA)),
        Press::Idle => None,
    };
    let radius = if round { rect.height / 2.0 } else { 0.0 };
    if let Some(color) = background {
        painter.scene.fill(rect, color, radius);
    }
    let color = match state {
        _ if closing => Rgba::hex(0xffffff),
        Press::Idle => theme.text_muted,
        _ => theme.text,
    };
    let glyph = match (round, icon) {
        (false, Icon::Close) => SQUARE_CROSS,
        (false, _) => SQUARE_GLYPH,
        (true, Icon::Close) => ROUND_CROSS,
        (true, _) => ROUND_GLYPH,
    };
    let size = painter.metrics.px(glyph);
    painter.icon(icon, centered(rect, size), color);
}
