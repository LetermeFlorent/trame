use crate::render::Scene;
use crate::text::TextSystem;
use crate::ui::chrome::{ChromeInput, Painter};
use crate::ui::hit::Hit;

const TOOLTIP_PAD_X: f32 = 8.0;
const TOOLTIP_BLUR: f32 = 6.0;

pub fn draw_tooltip(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let frame = input.frame;
    let (Some(Hit::RailInstance(id)), Some(rail), None) =
        (input.view.hover, &frame.rail, &frame.editor)
    else {
        return;
    };
    let Some(slot) = rail.slots.iter().find(|slot| slot.id == id) else {
        return;
    };
    let metrics = &frame.metrics;
    let label = text.layout(&slot.name, metrics.label());
    let pad = metrics.px(TOOLTIP_PAD_X);
    let Some(bubble) = rail.beside(metrics, id, (label.width + 2.0 * pad).ceil()) else {
        return;
    };
    let mut painter = Painter::new(scene, text, input);
    painter.panel(bubble, TOOLTIP_BLUR);
    painter.label(&label, bubble.x + pad, bubble, input.theme.text);
}
