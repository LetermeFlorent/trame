use super::target::DropMark;
use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::text::TextSystem;
use crate::ui::chrome::ChromeInput;
use crate::ui::frame::fit_label;
use crate::ui::metrics::UiMetrics;

const GHOST_ALPHA: f32 = 0.85;
const GHOST_OFFSET: f32 = 10.0;
const GHOST_BLUR: f32 = 8.0;
const OUTLINE_ALPHA: f32 = 0.6;

#[derive(Clone, Debug, PartialEq)]
pub struct DragView {
    pub label: String,
    pub pointer: Point,
    pub mark: Option<DropMark>,
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
    draw_ghost(scene, text, input, drag);
}

fn draw_ghost(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput, drag: &DragView) {
    let theme = input.theme;
    let metrics = input.frame.metrics;
    let padding = metrics.px(UiMetrics::TAB_PADDING);
    let widest = metrics.px(UiMetrics::TAB_MAX) - padding * 2.0;
    let label = fit_label(text, &drag.label, metrics.label(), widest);
    let offset = metrics.px(GHOST_OFFSET);
    let rect = Rect::new(
        (drag.pointer.x + offset).round(),
        (drag.pointer.y + offset).round(),
        (label.width + padding * 2.0).ceil(),
        metrics.px(UiMetrics::TAB_BAR),
    );
    let radius = metrics.px(UiMetrics::RADIUS);
    scene.shadow(rect, theme.shadow, radius, metrics.px(GHOST_BLUR));
    scene.fill(rect, theme.tab_active.with_alpha(GHOST_ALPHA), radius);
    scene.border(rect, theme.border_strong, radius, metrics.line());
    let baseline = (rect.y + (rect.height + label.ascent - label.descent) / 2.0).round();
    let pen = Point::new((rect.x + padding).round(), baseline);
    text.draw_layout(scene, &label, pen, theme.text);
}
