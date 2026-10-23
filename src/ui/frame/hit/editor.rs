use super::Hit;
use crate::geometry::{Point, Rect};
use crate::ui::frame::{EditorFrame, FormFrame, HostList, MessageFrame};

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

pub(super) fn hit_message(message: &MessageFrame, point: Point) -> Option<Hit> {
    let popup = &message.popup;
    if !popup.rect.contains(point) {
        return popup.backdrop.contains(point).then_some(Hit::Backdrop);
    }
    if popup.close.contains(point) {
        return Some(Hit::EditClose);
    }
    Some(
        message
            .button_at(point)
            .map_or(Hit::Dialog, Hit::FormButton),
    )
}

fn hit_dialog(editor: &EditorFrame, close: Rect, point: Point) -> Hit {
    if close.contains(point) {
        return Hit::EditClose;
    }
    if let Some(hit) = host_hit(editor.form.as_ref(), editor.list.as_ref(), point) {
        return hit;
    }
    if let Some(part) = editor
        .picker
        .as_ref()
        .and_then(|picker| picker.part_at(point))
    {
        return Hit::Picker(part);
    }
    if let Some(slot) = editor.slot_at(point) {
        return Hit::EditSwatch(slot.color);
    }
    editor
        .emblem_at(point)
        .map_or(Hit::EditStrip, |slot| Hit::EditEmblem(slot.emblem))
}

pub(super) fn host_hit(
    form: Option<&FormFrame>,
    list: Option<&HostList>,
    point: Point,
) -> Option<Hit> {
    let form_hit = form.and_then(|form| {
        form.field_at(point)
            .map(Hit::FormField)
            .or_else(|| form.button_at(point).map(Hit::FormButton))
    });
    form_hit.or_else(|| list?.control_at(point).map(Hit::List))
}
