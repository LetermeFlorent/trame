use super::Painter;
use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::text::TextSystem;
use crate::theme::{Swatch, Theme};
use crate::ui::frame::{EditorFrame, FrameLayout, SwatchSlot};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;
use crate::ui::paint::FrameView;

const STRIP_BLUR: f32 = 8.0;
const RING_GAP: f32 = 2.0;
const RING_RADIUS_EXTRA: f32 = 2.0;
const SLASH_INSET: f32 = 3.0;

pub fn draw_editor(
    scene: &mut Scene,
    text: &mut TextSystem,
    theme: &Theme,
    frame: &FrameLayout,
    view: &FrameView,
) {
    let Some(editor) = &frame.editor else {
        return;
    };
    let mut painter = Painter {
        scene,
        text,
        theme,
        metrics: frame.metrics,
        view,
    };
    draw_strip(&mut painter, editor);
    draw_field(&mut painter, editor);
}

fn draw_strip(painter: &mut Painter, editor: &EditorFrame) {
    let theme = painter.theme;
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let line = painter.metrics.line();
    let blur = painter.metrics.px(STRIP_BLUR);
    painter
        .scene
        .shadow(editor.strip, theme.shadow, radius, blur);
    painter.scene.fill(editor.strip, theme.chrome, radius);
    painter
        .scene
        .border(editor.strip, theme.border_strong, radius, line);
    for slot in &editor.slots {
        draw_slot(painter, slot, editor.selected);
    }
}

fn draw_slot(painter: &mut Painter, slot: &SwatchSlot, selected: Option<Swatch>) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    let radius = slot.rect.width / 2.0;
    match slot.color {
        Some(color) => painter.scene.fill(slot.rect, theme.swatch(color), radius),
        None => draw_empty(painter, slot.rect),
    }
    let hovered = painter.view.hover == Some(Hit::EditSwatch(slot.color));
    let ring = match () {
        _ if slot.color == selected => Some(theme.text),
        _ if hovered => Some(theme.text_muted),
        _ => None,
    };
    if let Some(color) = ring {
        let gap = painter.metrics.px(RING_GAP);
        let outer = slot.rect.inset(-gap);
        let outer_radius = radius + painter.metrics.px(RING_RADIUS_EXTRA);
        painter.scene.border(outer, color, outer_radius, line);
    }
}

fn draw_empty(painter: &mut Painter, rect: Rect) {
    let theme = painter.theme;
    let line = painter.metrics.line();
    painter
        .scene
        .border(rect, theme.text_faint, rect.width / 2.0, line);
    let inset = painter.metrics.px(SLASH_INSET);
    let from = Point::new(rect.x + inset, rect.bottom() - inset);
    let to = Point::new(rect.right() - inset, rect.y + inset);
    painter.scene.line(from, to, line, theme.text_faint);
}

fn draw_field(painter: &mut Painter, editor: &EditorFrame) {
    let theme = painter.theme;
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let line = painter.metrics.line();
    painter.scene.fill(editor.field, theme.terminal, radius);
    painter
        .scene
        .border(editor.field, theme.accent, radius, line);
    painter.scene.push_clip(editor.text_area);
    painter.label(&editor.label, editor.text_x, editor.field, theme.text);
    painter.scene.fill(editor.caret, theme.text, 0.0);
    painter.scene.pop_clip();
}
