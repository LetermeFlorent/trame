use super::LoaderView;
use crate::geometry::Point;
use crate::render::Scene;
use crate::text::{TextLayout, TextSystem};
use crate::theme::Rgba;
use crate::ui::chrome::ChromeInput;

pub(super) fn draw_message(
    scene: &mut Scene,
    text: &mut TextSystem,
    input: &ChromeInput,
    loader: &LoaderView,
    top: Point,
) {
    let style = input.frame.metrics.label();
    let color = input.theme.text_muted;
    let leaving = loader.previous.filter(|_| loader.blend < 1.0);
    if let Some(previous) = leaving {
        let layout = text.layout(previous, style);
        scene.push_opacity(1.0 - loader.blend);
        draw_centered(scene, text, &layout, top, color);
        scene.pop_opacity();
    }
    let layout = text.layout(loader.message, style);
    scene.push_opacity(leaving.map_or(1.0, |_| loader.blend));
    draw_centered(scene, text, &layout, top, color);
    scene.pop_opacity();
}

pub(super) fn draw_centered(
    scene: &mut Scene,
    text: &mut TextSystem,
    layout: &TextLayout,
    top: Point,
    color: Rgba,
) {
    let pen = Point::new(
        (top.x - layout.width / 2.0).round(),
        (top.y + layout.ascent).round(),
    );
    text.draw_layout(scene, layout, pen, color);
}
