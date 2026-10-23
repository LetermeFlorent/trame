use super::Hit;
use crate::geometry::{Point, Rect};
use crate::ui::frame::EditorFrame;

pub(super) fn hit_editor(editor: &EditorFrame, point: Point) -> Option<Hit> {
    let Some(popup) = &editor.popup else {
        let field = editor.field.as_ref();
        return field
            .filter(|field| field.rect.contains(point))
            .map(|_| Hit::EditField);
    };
    match popup.rect.contains(point) {
        true => Some(hit_dialog(editor, popup.close, point)),
        false => popup.backdrop.contains(point).then_some(Hit::Backdrop),
    }
}

fn hit_dialog(editor: &EditorFrame, close: Rect, point: Point) -> Hit {
    if close.contains(point) {
        return Hit::EditClose;
    }
    if let Some(form) = editor
        .form
        .as_ref()
        .filter(|form| form.rect.contains(point))
    {
        return match form.field_at(point) {
            Some(field) => Hit::FormField(field),
            None if form.connect.contains(point) => Hit::FormConnect,
            None => Hit::EditStrip,
        };
    }
    if let Some(slot) = editor.slot_at(point) {
        return Hit::EditSwatch(slot.color);
    }
    if let Some(slot) = editor.emblem_at(point) {
        return Hit::EditEmblem(slot.emblem);
    }
    editor.host_at(point).map_or(Hit::EditStrip, Hit::EditHost)
}
