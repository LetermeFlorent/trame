use crate::ui::chrome::{Icon, Painter};
use crate::ui::frame::PopupFrame;
use crate::ui::hit::Hit;

const PANEL_BLUR: f32 = 8.0;
const BACKDROP_ALPHA: f32 = 0.35;

pub(super) fn draw_popup(painter: &mut Painter, popup: &PopupFrame, close: Hit) {
    let theme = painter.theme;
    let shade = theme.shadow.with_alpha(BACKDROP_ALPHA);
    painter.scene.fill(popup.backdrop, shade, 0.0);
    painter.panel(popup.rect, PANEL_BLUR);
    painter.setting_line(&popup.title, theme.text);
    painter.button(popup.close, close, Icon::Close);
}
