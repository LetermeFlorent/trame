use crate::input::{Keymap, Overrides};
use crate::locale::KeyStrings;
use crate::ui::Platform;
use crate::ui::frame::fit_label;
use crate::ui::frame::sidebar::settings::column::{Column, LINE_HEIGHT, LINE_INSET};
use crate::ui::frame::sidebar::settings::{
    ChoiceRow, HostRow, ListSpec, SettingControl, SettingLine, SettingsRequest,
};

use super::editor::editor_page;
use super::{KeyControl, KeyDetail, ShortcutsPage, ShortcutsView, key_label};

const NAME_SHARE: f32 = 0.5;

pub(super) struct Book<'a> {
    pub(super) strings: &'static KeyStrings,
    keymap: Keymap,
    overrides: &'a Overrides,
    platform: Platform,
}

impl Book<'_> {
    pub(super) fn labels(&self, action: usize) -> Vec<String> {
        self.keymap
            .keys(action)
            .iter()
            .map(|combo| key_label(combo, self.strings, self.platform))
            .collect()
    }

    pub(super) fn changed(&self, action: usize) -> bool {
        self.overrides.keys(action).is_some()
    }
}

pub(in crate::ui::frame::sidebar::settings) fn shortcuts_page(
    column: &mut Column,
    request: &SettingsRequest,
    height: f32,
) -> ShortcutsPage {
    let view = request.shortcuts.unwrap_or_default();
    let platform = column.metrics.platform;
    let overrides = &request.preferences.shortcuts;
    let book = Book {
        strings: request.strings.keys,
        keymap: Keymap::new(platform, overrides),
        overrides,
        platform,
    };
    let bottom = column.y + height;
    match view.action {
        Some(action) => editor_page(column, &book, view, action),
        None => list_page(column, &book, view, bottom),
    }
}

pub(super) fn empty(header: ChoiceRow) -> ShortcutsPage {
    ShortcutsPage {
        header,
        list: None,
        details: Vec::new(),
        keys: Vec::new(),
        hint: None,
        capturing: false,
        footer: None,
        notice: None,
    }
}

fn list_page(column: &mut Column, book: &Book, view: ShortcutsView, bottom: f32) -> ShortcutsPage {
    let strings = book.strings;
    let caption = column.caption(strings.tab);
    let reset = [(
        strings.all_default,
        SettingControl::Keys(KeyControl::ResetAll),
        false,
    )];
    let shown = usize::from(!book.overrides.is_empty());
    let header = column.choices_on(caption, &reset[..shown], None);
    let room = (bottom - column.y) / column.metrics.px(LINE_HEIGHT);
    let labels: Vec<String> = strings
        .actions
        .iter()
        .map(|name| name.to_string())
        .collect();
    let spec = ListSpec {
        labels: &labels,
        first: view.first,
        shown: room.floor().max(1.0) as usize,
        pickable: true,
        forgets: false,
    };
    let mut list = column.host_list(&spec);
    let details = list
        .rows
        .iter_mut()
        .map(|row| detail(column, book, row))
        .collect();
    ShortcutsPage {
        list: Some(list),
        details,
        ..empty(header)
    }
}

fn detail(column: &mut Column, book: &Book, row: &mut HostRow) -> KeyDetail {
    let inset = column.metrics.px(LINE_INSET);
    let style = column.metrics.label();
    let room = row.change.x - row.line.label_x - 2.0 * inset;
    let name = book.strings.actions[row.index];
    row.line.label = fit_label(column.text, name, style, room * NAME_SHARE);
    let keys = book.labels(row.index);
    let text = match keys.is_empty() {
        true => book.strings.no_key.to_string(),
        false => keys.join(", "),
    };
    let label = fit_label(
        column.text,
        &text,
        style,
        room - row.line.label.width - inset,
    );
    let line = SettingLine {
        rect: row.line.rect,
        label_x: row.change.x - inset - label.width,
        label,
    };
    KeyDetail {
        line,
        changed: book.changed(row.index),
    }
}
