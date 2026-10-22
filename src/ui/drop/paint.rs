use super::ghost::{Ghost, paint_ghost};
use super::target::DropMark;
use crate::geometry::Point;
use crate::render::Scene;
use crate::text::TextSystem;
use crate::ui::chrome::ChromeInput;
use crate::ui::metrics::UiMetrics;

const OUTLINE_ALPHA: f32 = 0.6;

#[derive(Clone, Debug, PartialEq)]
pub struct DragView {
    pub label: String,
    pub pointer: Point,
    pub mark: Option<DropMark>,
    pub ghost: bool,
}

pub fn draw_drag(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput, drag: &DragView) {
    let theme = input.theme;
    let metrics = input.frame.metrics;
    match drag.mark {
        Some(DropMark::Area(rect)) => {
            let radius = metrics.px(UiMetrics::RADIUS);
            scene.fill(rect, theme.drop_target, radius);
            scene.border(
                rect,
                theme.accent.with_alpha(OUTLINE_ALPHA),
                radius,
                metrics.line(),
            );
        }
        Some(DropMark::Marker(rect)) => scene.fill(rect, theme.accent, 0.0),
        None => {}
    }
    if drag.ghost {
        draw_ghost(scene, text, input, drag);
    }
}

fn draw_ghost(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput, drag: &DragView) {
    let metrics = input.frame.metrics;
    let ghost = Ghost {
        label: &drag.label,
        pointer: drag.pointer,
    };
    paint_ghost(scene, text, input.theme, metrics, ghost);
}
