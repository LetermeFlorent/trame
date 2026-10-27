use crate::ui::chrome::overlay::connect::draw_host_list;
use crate::ui::chrome::{Icon, Painter};
use crate::ui::frame::{KeyControl, SettingControl, ShortcutsPage};
use crate::ui::hit::Hit;

use super::controls::{draw_chip, draw_choices};

pub(super) fn draw_shortcuts(painter: &mut Painter, page: &ShortcutsPage) {
    let theme = painter.theme;
    painter.setting_line(&page.header.line, theme.text_faint);
    for chip in &page.header.chips {
        draw_chip(painter, chip);
    }
    if let Some(list) = &page.list {
        draw_host_list(painter, list);
    }
    for detail in &page.details {
        let color = match detail.changed {
            true => theme.accent,
            false => theme.text_muted,
        };
        painter.setting_line(&detail.line, color);
    }
    for key in &page.keys {
        painter.setting_line(&key.line, theme.text);
        let remove = Hit::Setting(SettingControl::Keys(KeyControl::Remove(key.index)));
        painter.small_button(key.remove, remove, Icon::Close);
    }
    if let Some(hint) = &page.hint {
        let color = match page.capturing {
            true => theme.accent,
            false => theme.text_muted,
        };
        painter.setting_line(hint, color);
    }
    if let Some(footer) = &page.footer {
        draw_choices(painter, footer);
    }
    if let Some(notice) = &page.notice {
        painter.setting_line(notice, theme.text);
    }
}
