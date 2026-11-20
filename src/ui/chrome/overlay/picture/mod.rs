mod draw;
mod preview;

pub use draw::draw_picture;
pub use preview::{
    HoverPreviewFrame, draw_hover_preview, hover_preview_limit, place_hover_preview,
};

use crate::files::pictures::fit_size;
use crate::geometry::Rect;
use crate::text::TextSystem;
use crate::ui::metrics::UiMetrics;

const PICTURE_MARGIN: f32 = 32.0;
const CLOSE_SIZE: f32 = 28.0;
const CLOSE_PAD: f32 = 12.0;
const OPEN_GAP: f32 = 8.0;
const OPEN_PAD_X: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureButton {
    Close,
    Open,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PictureFrame {
    pub size: (u32, u32),
    pub rect: Rect,
    pub open: Option<Rect>,
    pub open_label: Option<&'static str>,
    pub hovered: Option<PictureButton>,
}

impl PictureFrame {
    pub fn centered(bounds: Rect, width: u32, height: u32) -> Self {
        let (w, h) = (width as f32, height as f32);
        Self {
            size: (width, height),
            rect: Rect::new(
                bounds.centered_x(w).round(),
                bounds.centered_y(h).round(),
                w,
                h,
            ),
            open: None,
            open_label: None,
            hovered: None,
        }
    }

    pub fn with_open(mut self, rect: Option<Rect>, label: Option<&'static str>) -> Self {
        self.open = rect;
        self.open_label = label;
        self
    }
}

pub fn picture_area_limit(bounds: Rect, metrics: &UiMetrics) -> (u32, u32) {
    let area = bounds.inset(metrics.px(PICTURE_MARGIN));
    (
        area.width.floor().max(0.0) as u32,
        area.height.floor().max(0.0) as u32,
    )
}

pub fn place_picture(bounds: Rect, metrics: &UiMetrics, width: u32, height: u32) -> PictureFrame {
    let (max_w, max_h) = picture_area_limit(bounds, metrics);
    let (w, h) = fit_size(width, height, max_w, max_h);
    PictureFrame::centered(bounds, w, h)
}

pub fn picture_close_rect(bounds: Rect, title_bottom: f32, metrics: &UiMetrics) -> Rect {
    let size = metrics.px(CLOSE_SIZE);
    let pad = metrics.px(CLOSE_PAD);
    let top = title_bottom + pad;
    Rect::new(
        (bounds.right() - pad - size).max(bounds.x),
        top.min((bounds.bottom() - size).max(bounds.y)),
        size,
        size,
    )
    .snapped()
}

pub fn place_picture_open(
    text: &mut TextSystem,
    bounds: Rect,
    title_bottom: f32,
    metrics: &UiMetrics,
    label: &str,
) -> Option<Rect> {
    let close = picture_close_rect(bounds, title_bottom, metrics);
    let pad = metrics.px(CLOSE_PAD);
    let gap = metrics.px(OPEN_GAP);
    let pad_x = metrics.px(OPEN_PAD_X);
    let max_w = close.x - gap - (bounds.x + pad);
    if max_w < close.height {
        return None;
    }
    let layout = text.layout(label, metrics.label());
    let width = (layout.width.ceil() + 2.0 * pad_x.ceil())
        .min(max_w.floor())
        .max(close.height);
    Some(
        Rect::new(
            (close.x - gap - width).floor(),
            close.y,
            width,
            close.height,
        )
        .snapped(),
    )
}
