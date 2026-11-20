use super::{OPEN_PAD_X, PICTURE_MARGIN, PictureButton, PictureFrame, picture_close_rect};
use crate::geometry::Rect;
use crate::render::Scene;
use crate::text::TextSystem;
use crate::theme::Rgba;
use crate::ui::chrome::painter::{Painter, centered};
use crate::ui::chrome::{ChromeInput, Icon};
use crate::ui::frame::fit_label;
use crate::ui::metrics::UiMetrics;

const SCRIM_ALPHA: f32 = 0.72;
const CLOSE_ALPHA: f32 = 0.9;
const HOVER_BLEND: f32 = 0.35;

pub fn draw_picture(
    scene: &mut Scene,
    text: &mut TextSystem,
    input: &ChromeInput,
    picture: Option<PictureFrame>,
) {
    let Some(picture) = picture else {
        return;
    };
    let bounds = input.frame.bounds;
    let metrics = &input.frame.metrics;
    let theme = input.theme;
    let area = bounds.inset(metrics.px(PICTURE_MARGIN));
    scene.fill(bounds, Rgba::BLACK.with_alpha(SCRIM_ALPHA), 0.0);
    if !picture.rect.is_empty() {
        scene.push_clip(area);
        scene.picture(picture.rect);
        scene.pop_clip();
    }
    let radius = metrics.px(UiMetrics::RADIUS);
    let pad_x = metrics.px(OPEN_PAD_X);
    let mut painter = Painter::new(scene, text, input);
    if let (Some(open), Some(label)) = (picture.open, picture.open_label) {
        let hovered = picture.hovered == Some(PictureButton::Open);
        draw_overlay_button(&mut painter, open, radius, hovered);
        let room = (open.width - 2.0 * pad_x).max(0.0);
        let layout = fit_label(painter.text, label, metrics.label(), room);
        let x = open.centered_x(layout.width);
        painter.label(&layout, x, open, theme.text);
    }
    let close = picture_close_rect(bounds, input.frame.title_bar.rect.bottom(), metrics);
    draw_overlay_button(
        &mut painter,
        close,
        radius,
        picture.hovered == Some(PictureButton::Close),
    );
    painter.icon(
        Icon::Close,
        centered(close, metrics.px(UiMetrics::ICON)),
        theme.text,
    );
}

fn draw_overlay_button(painter: &mut Painter, rect: Rect, radius: f32, hovered: bool) {
    let theme = painter.theme;
    let fill = match hovered {
        true => theme
            .chrome
            .mix(theme.hover, HOVER_BLEND)
            .with_alpha(CLOSE_ALPHA),
        false => theme.chrome.with_alpha(CLOSE_ALPHA),
    };
    painter.scene.fill(rect, fill, radius);
    painter
        .scene
        .border(rect, theme.border_strong, radius, painter.metrics.line());
}
