use crate::geometry::Rect;
use crate::render::Scene;
use crate::text::{TextLayout, TextSystem};
use crate::ui::chrome::{ChromeInput, Icon, Painter};
use crate::ui::frame::PopupFrame;
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

use super::ON_ACCENT;
use super::connect::draw_form;

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
    let color = match button.filled {
        true => {
            painter.shaded_fill(rect, button.hit, theme.accent, radius);
            ON_ACCENT
        }
        false => {
            let stroke = painter.metrics.px(BUTTON_STROKE);
            painter.scene.border(rect, theme.border, radius, stroke);
            let glow = painter.press_fill(rect, button.hit, radius);
            match button.faint && glow == 0.0 {
                true => theme.text_faint,
                false => theme.muted_text(glow),
            }
        }
    };
    let x = rect.centered_x(button.label.width);
    painter.label(button.label, x, rect, color);
}

pub fn draw_confirm(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let Some(confirm) = &input.frame.confirm else {
        return;
    };
    let mut painter = Painter::new(scene, text, input);
    draw_popup(&mut painter, &confirm.popup, Hit::EditClose);
    painter.setting_line(&confirm.message, painter.theme.text_muted);
    draw_form(&mut painter, &confirm.form);
}
