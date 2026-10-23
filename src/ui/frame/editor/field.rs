use std::sync::Arc;

use super::anchor::LabelSpot;
use super::clamp_x;
use crate::geometry::Rect;
use crate::input::TextField;
use crate::text::{TextLayout, TextStyle, TextSystem};
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
    pub selection: Option<Rect>,
    pub boxed: bool,
}

struct FieldArea {
    rect: Rect,
    text_area: Rect,
    style: TextStyle,
    boxed: bool,
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
    let area = FieldArea {
        rect,
        text_area: rect.inset_by(metrics.px(TEXT_PAD), 0.0),
        style: metrics.label(),
        boxed: true,
    };
    lay_text(text, metrics, field, area)
}

pub(super) fn place_inline(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    anchor: Rect,
    spot: LabelSpot,
    field: &TextField,
) -> FieldFrame {
    let right = spot.right.max(spot.x);
    let area = FieldArea {
        rect: anchor,
        text_area: Rect::from_edges(spot.x, anchor.y, right, anchor.bottom()),
        style: spot.style,
        boxed: false,
    };
    lay_text(text, metrics, field, area)
}

fn lay_text(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    field: &TextField,
    area: FieldArea,
) -> FieldFrame {
    let FieldArea {
        rect,
        text_area,
        style,
        boxed,
    } = area;
    let label = text.layout(field.text(), style);
    let caret_x = text.layout(field.before_caret(), style).width;
    let caret_width = metrics.line();
    let offset = (caret_x + caret_width - text_area.width).max(0.0);
    let text_x = text_area.x - offset;
    let height = (label.ascent + label.descent).round();
    let top = (rect.center().y - height / 2.0).round();
    let caret = Rect::new((text_x + caret_x).round(), top, caret_width, height);
    let selection = field
        .is_selected()
        .then(|| Rect::new(text_x.round(), top, label.width.ceil(), height));
    FieldFrame {
        rect,
        text_area,
        label,
        text_x,
        caret,
        selection,
        boxed,
    }
}
