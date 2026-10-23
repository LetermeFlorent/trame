use super::draw_backdrop;
use crate::geometry::Rect;
use crate::theme::{Rgba, Swatch};
use crate::ui::chrome::{Icon, Painter, centered};
use crate::ui::frame::{FooterButton, FooterFrame};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

const ON_UPDATE: Rgba = Rgba::hex(0x1b1b1b);

pub(in crate::ui::chrome) fn draw_footer(
    painter: &mut Painter,
    footer: &FooterFrame,
    open: bool,
    reveal: f32,
) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    let rect = footer.rect;
    painter.rule(
        Rect::new(rect.x, rect.y, rect.width - line, line),
        theme.border,
    );
    if let Some(update) = &footer.update {
        draw_update(painter, update, reveal);
    }
    let settings = &footer.settings;
    draw_backdrop(painter, settings.rect, Hit::Settings, open, None);
    let color = match open {
        true => theme.text,
        false => theme.muted_text(painter.glow(Hit::Settings)),
    };
    draw_content(painter, settings, Icon::Gear, color, reveal);
}

fn draw_update(painter: &mut Painter, button: &FooterButton, reveal: f32) {
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let orange = painter.theme.swatch(Swatch::Orange);
    painter.shaded_fill(button.rect, Hit::Update, orange, radius);
    draw_content(painter, button, Icon::Update, ON_UPDATE, reveal);
}

fn draw_content(
    painter: &mut Painter,
    button: &FooterButton,
    icon: Icon,
    color: Rgba,
    reveal: f32,
) {
    let size = painter.metrics.px(UiMetrics::ICON);
    painter.icon(icon, centered(button.icon, size), color);
    if reveal > 0.0 {
        painter.scene.push_clip(button.rect);
        painter.scene.push_opacity(reveal);
        painter.label(&button.label, button.label_x, button.rect, color);
        painter.scene.pop_opacity();
        painter.scene.pop_clip();
    }
}
