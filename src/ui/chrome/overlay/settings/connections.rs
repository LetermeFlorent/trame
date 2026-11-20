use crate::ui::chrome::Painter;
use crate::ui::chrome::overlay::connect::{draw_form, draw_host_list};
use crate::ui::frame::ConnectionsPage;

use super::controls::draw_header;
use super::draw_toggle_row;

pub(super) fn draw_connections(painter: &mut Painter, page: &ConnectionsPage) {
    let theme = painter.theme;
    draw_header(painter, &page.header);
    if let Some(toggle) = &page.toggle {
        draw_toggle_row(painter, toggle);
    }
    if let Some(hint) = &page.hint {
        painter.setting_line(hint, theme.text_muted);
    }
    if let Some(list) = &page.list {
        draw_host_list(painter, list);
    }
    if let Some(form) = &page.form {
        draw_form(painter, form);
    }
}
