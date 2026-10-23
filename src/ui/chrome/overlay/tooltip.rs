use std::sync::Arc;

use crate::geometry::Rect;
use crate::render::Scene;
use crate::text::{TextLayout, TextSystem};
use crate::ui::chrome::{ChromeInput, Painter};
use crate::ui::frame::FrameLayout;
use crate::ui::hit::Hit;

const TOOLTIP_PAD_X: f32 = 8.0;
const TOOLTIP_BLUR: f32 = 6.0;

pub fn draw_tooltip(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let frame = input.frame;
    let (Some(hover), Some(rail), None) = (input.view.hover, &frame.rail, &frame.editor) else {
        return;
    };
    let Some((label, anchor)) = tip(text, frame, hover) else {
        return;
    };
    let metrics = &frame.metrics;
    let pad = metrics.px(TOOLTIP_PAD_X);
    let bubble = rail.next_to(metrics, anchor, (label.width + 2.0 * pad).ceil());
    let mut painter = Painter::new(scene, text, input);
    painter.panel(bubble, TOOLTIP_BLUR);
    painter.label(&label, bubble.x + pad, bubble, input.theme.text);
}

fn tip(text: &mut TextSystem, frame: &FrameLayout, hover: Hit) -> Option<(Arc<TextLayout>, Rect)> {
    match hover {
        Hit::RailInstance(id) => {
            let rail = frame.rail.as_ref()?;
            let slot = rail.slots.iter().find(|slot| slot.id == id)?;
            Some((text.layout(&slot.name, frame.metrics.label()), slot.rect))
        }
        Hit::Settings if frame.sidebar.is_none() => {
            let footer = &frame.footer;
            Some((footer.label.clone(), footer.settings))
        }
        _ => None,
    }
}
