use crate::ui::chrome::Painter;
use crate::ui::chrome::overlay::connect::{draw_form, draw_host_list};
use crate::ui::frame::ConnectionsPage;

use super::controls::draw_chip;

pub(super) fn draw_connections(painter: &mut Painter, page: &ConnectionsPage) {
    let theme = painter.theme;
    painter.setting_line(&page.header.line, theme.text_faint);
    for chip in &page.header.chips {
        draw_chip(painter, chip);
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
