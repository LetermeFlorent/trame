use crate::geometry::Rect;
use crate::render::Scene;
use crate::text::TextSystem;
use crate::theme::Rgba;
use crate::ui::chrome::{ChromeInput, Painter};
use crate::ui::metrics::UiMetrics;

const PREVIEW_MAX_W: f32 = 220.0;
const PREVIEW_MAX_H: f32 = 160.0;
const PREVIEW_PAD: f32 = 6.0;
const PREVIEW_GAP: f32 = 6.0;
const PREVIEW_MARGIN: f32 = 8.0;
const PREVIEW_BLUR: f32 = 6.0;

#[derive(Clone, Copy, Debug)]
pub struct HoverPreviewFrame<'a> {
    pub job: u64,
    pub anchor: Rect,
    pub size: (u32, u32),
    pub pixels: &'a [u8],
}

pub fn hover_preview_limit(metrics: &UiMetrics, max_side: u32) -> (u32, u32) {
    let cap = max_side.max(1);
    (
        (metrics.px(PREVIEW_MAX_W).round().max(1.0) as u32).min(cap),
        (metrics.px(PREVIEW_MAX_H).round().max(1.0) as u32).min(cap),
    )
}

pub fn place_hover_preview(
    bounds: Rect,
    title_bottom: f32,
    anchor: Rect,
    metrics: &UiMetrics,
    img_size: (u32, u32),
) -> (Rect, Rect) {
    let pad = metrics.px(PREVIEW_PAD);
    let gap = metrics.px(PREVIEW_GAP);
    let margin = metrics.px(PREVIEW_MARGIN);
    let card_w = img_size.0 as f32 + 2.0 * pad;
    let card_h = img_size.1 as f32 + 2.0 * pad;
    let min_x = bounds.x + margin;
    let max_x = (bounds.right() - margin - card_w).max(min_x);
    let x = anchor.x.clamp(min_x, max_x).round();
    let min_y = title_bottom + margin;
    let max_y = (bounds.bottom() - margin - card_h).max(min_y);
    let above = anchor.y - gap - card_h;
    let y = if above >= min_y {
        above.round()
    } else {
        (anchor.bottom() + gap).clamp(min_y, max_y).round()
    };
    let card = Rect::new(x, y, card_w, card_h).snapped();
    let image = Rect::new(
        card.x + pad,
        card.y + pad,
        img_size.0 as f32,
        img_size.1 as f32,
    );
    (card, image)
}

pub fn draw_hover_preview(
    scene: &mut Scene,
    text: &mut TextSystem,
    input: &ChromeInput,
    preview: Option<HoverPreviewFrame>,
) {
    let Some(preview) = preview.filter(|p| p.size.0 > 0 && p.size.1 > 0) else {
        return;
    };
    let Some(region) =
        text.hover_image(preview.job, preview.size.0, preview.size.1, preview.pixels)
    else {
        return;
    };
    let (card, image) = place_hover_preview(
        input.frame.bounds,
        input.frame.title_bar.rect.bottom(),
        preview.anchor,
        &input.frame.metrics,
        preview.size,
    );
    let mut painter = Painter::new(scene, text, input);
    painter.panel(card, PREVIEW_BLUR);
    painter.scene.glyph(region, image.x, image.y, Rgba::WHITE);
}
