mod form;
mod list;

use crate::ui::chrome::Painter;
use crate::ui::frame::EditorFrame;
pub(super) use form::draw_form;
pub(super) use list::draw_host_list;

pub(super) fn draw_connect_dialog(painter: &mut Painter, editor: &EditorFrame) {
    let faint = painter.theme.text_faint;
    for section in &editor.sections {
        painter.setting_line(section, faint);
    }
    if let Some(list) = &editor.list {
        draw_host_list(painter, list);
    }
    if let Some(form) = &editor.form {
        draw_form(painter, form);
    }
}
