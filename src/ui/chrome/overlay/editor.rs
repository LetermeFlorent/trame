use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::text::TextSystem;
use crate::theme::{Emblem, Swatch};
use crate::ui::chrome::{ChromeInput, Icon, Painter};
use crate::ui::frame::{EditorFrame, EmblemSlot, FieldFrame, SwatchSlot};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

use super::form::draw_form;

pub(super) const STRIP_BLUR: f32 = 8.0;
const RING_GAP: f32 = 2.0;
const RING_RADIUS_EXTRA: f32 = 2.0;
const SLASH_INSET: f32 = 3.0;

pub fn draw_editor(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let Some(editor) = &input.frame.editor else {
        return;
    };
    let mut painter = Painter::new(scene, text, input);
    if !editor.strip.is_empty() {
        draw_strip(&mut painter, editor);
    }
    if let Some(field) = &editor.field {
        draw_field(&mut painter, field, true);
    }
    if let Some(form) = &editor.form {
        draw_form(&mut painter, form);
    }
}

fn draw_strip(painter: &mut Painter, editor: &EditorFrame) {
    painter.panel(editor.strip, STRIP_BLUR);
    for slot in &editor.slots {
        draw_slot(painter, slot, editor.selected);
    }
    for slot in &editor.emblems {
        draw_emblem(painter, slot, editor.emblem);
    }
}

fn draw_emblem(painter: &mut Painter, slot: &EmblemSlot, selected: Option<Emblem>) {
    let theme = painter.theme;
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let chosen = selected == Some(slot.emblem);
    if painter.view.hover == Some(Hit::EditEmblem(slot.emblem)) {
        painter.scene.fill(slot.rect, theme.hover, radius);
    }
    if chosen {
        let line = painter.metrics.line();
        painter.scene.border(slot.rect, theme.text, radius, line);
    }
    let color = match chosen {
        true => theme.text,
        false => theme.text_muted,
    };
    painter.icon(Icon::Emblem(slot.emblem), slot.icon, color);
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

pub(super) fn draw_field(painter: &mut Painter, field: &FieldFrame, focused: bool) {
    let theme = painter.theme;
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let line = painter.metrics.line();
    let edge = match focused {
        true => theme.accent,
        false => theme.border_strong,
    };
    painter.scene.fill(field.rect, theme.terminal, radius);
    painter.scene.border(field.rect, edge, radius, line);
    painter.scene.push_clip(field.text_area);
    painter.label(&field.label, field.text_x, field.rect, theme.text);
    if focused {
        painter.scene.fill(field.caret, theme.text, 0.0);
    }
    painter.scene.pop_clip();
}
