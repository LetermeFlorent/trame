use crate::geometry::Rect;
use crate::text::TextLayout;
use crate::ui::chrome::{Icon, Painter};
use crate::ui::frame::PopupFrame;
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

use super::ON_ACCENT;

const PANEL_BLUR: f32 = 8.0;
const BACKDROP_ALPHA: f32 = 0.35;
const BUTTON_STROKE: f32 = 1.0;

pub(super) struct LabelButton<'a> {
    pub(super) rect: Rect,
    pub(super) label: &'a TextLayout,
    pub(super) hit: Hit,
    pub(super) filled: bool,
    pub(super) faint: bool,
}

pub(super) fn draw_popup(painter: &mut Painter, popup: &PopupFrame, close: Hit) {
    let theme = painter.theme;
    let shade = theme.shadow.with_alpha(BACKDROP_ALPHA);
    painter.scene.fill(popup.backdrop, shade, 0.0);
    painter.panel(popup.rect, PANEL_BLUR);
    painter.setting_line(&popup.title, theme.text);
    painter.button(popup.close, close, Icon::Close);
}

pub(super) fn draw_label_button(painter: &mut Painter, button: &LabelButton) {
    let theme = painter.theme;
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let rect = button.rect;
    match button.filled {
        true => painter.scene.fill(rect, theme.accent, radius),
        false => {
            let stroke = painter.metrics.px(BUTTON_STROKE);
            painter.scene.border(rect, theme.border, radius, stroke);
        }
    }
    let glow = painter.press_fill(rect, button.hit, radius);
    let color = match () {
        _ if button.filled => ON_ACCENT,
        _ if button.faint && glow == 0.0 => theme.text_faint,
        _ => theme.muted_text(glow),
    };
    let x = rect.centered_x(button.label.width);
    painter.label(button.label, x, rect, color);
}
