use std::sync::Arc;

use super::clamp_x;
use crate::geometry::Rect;
use crate::input::TextField;
use crate::text::{TextLayout, TextSystem};
use crate::ui::metrics::UiMetrics;

const FIELD_INSET_Y: f32 = 2.0;
const TEXT_PAD: f32 = 6.0;

#[derive(Clone, Debug)]
pub struct FieldFrame {
    pub rect: Rect,
    pub text_area: Rect,
    pub label: Arc<TextLayout>,
    pub text_x: f32,
    pub caret: Rect,
}

pub(in crate::ui::frame) fn place_field(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    anchor: Rect,
    bounds: Rect,
    field: &TextField,
) -> FieldFrame {
    let width = anchor.width.min(bounds.width);
    let inset = metrics.px(FIELD_INSET_Y);
    let x = clamp_x(anchor.x, width, bounds);
    let rect = Rect::new(x, anchor.y + inset, width, anchor.height - 2.0 * inset);
    let text_area = rect.inset_by(metrics.px(TEXT_PAD), 0.0);
    let style = metrics.label();
    let label = text.layout(field.text(), style);
    let caret_x = text.layout(field.before_caret(), style).width;
    let caret_width = metrics.line();
    let offset = (caret_x + caret_width - text_area.width).max(0.0);
    let text_x = text_area.x - offset;
    let caret_height = (label.ascent + label.descent).round();
    let caret = Rect::new(
        (text_x + caret_x).round(),
        (rect.center().y - caret_height / 2.0).round(),
        caret_width,
        caret_height,
    );
    FieldFrame {
        rect,
        text_area,
        label,
        text_x,
        caret,
    }
}
