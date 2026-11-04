use crate::input::MAX_KEYS;
use crate::locale::{ACTION_SLOT, KeyStrings};
use crate::ui::frame::sidebar::settings::chips::Choice;
use crate::ui::frame::sidebar::settings::column::{Column, LINE_HEIGHT, SECTION_GAP};
use crate::ui::frame::sidebar::settings::{ChoiceRow, SettingControl};

use super::place::{Book, empty};
use super::{KeyControl, KeyLine, KeyNotice, ShortcutsPage, ShortcutsView};

pub(super) fn editor_page(
    column: &mut Column,
    book: &Book,
    view: ShortcutsView,
    action: usize,
) -> ShortcutsPage {
    let strings = book.strings;
    let caption = column.caption(strings.actions[action]);
    let back = (strings.back, SettingControl::Keys(KeyControl::Back), false);
    let header = column.choices_on(caption, &[back], None);
    let style = column.metrics.label();
    let keys: Vec<KeyLine> = book
        .labels(action)
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let line = column.line(label, LINE_HEIGHT, style);
            let remove = column.end_button(&line);
            KeyLine {
                line,
                remove,
                index,
            }
        })
        .collect();
    let hint = match () {
        _ if view.capturing => Some(strings.press_keys),
        _ if keys.is_empty() => Some(strings.no_key),
        _ => None,
    };
    let hint = hint.map(|text| column.line(text, LINE_HEIGHT, style));
    column.y += column.metrics.px(SECTION_GAP);
    let add = !view.capturing && keys.len() < MAX_KEYS;
    let footer = footer(column, strings, add, book.changed(action));
    let notice = view.notice.map(|notice| {
        let text = notice_text(strings, notice);
        column.line(&text, LINE_HEIGHT, style)
    });
    ShortcutsPage {
        keys,
        hint,
        capturing: view.capturing,
        footer,
        notice,
        ..empty(header)
    }
}

fn footer(column: &mut Column, strings: &KeyStrings, add: bool, reset: bool) -> Option<ChoiceRow> {
    let choices: Vec<Choice> = [
        (add, strings.add_key, KeyControl::Add),
        (reset, strings.default, KeyControl::Reset),
    ]
    .into_iter()
    .filter(|(shown, _, _)| *shown)
    .map(|(_, label, control)| (label, SettingControl::Keys(control), false))
    .collect();
    (!choices.is_empty()).then(|| column.choice_row("", &choices, None))
}

fn notice_text(strings: &KeyStrings, notice: KeyNotice) -> String {
    match notice {
        KeyNotice::Taken(owner) => strings
            .taken_by
            .replace(ACTION_SLOT, strings.actions[owner]),
        KeyNotice::Typing => strings.typing.to_string(),
        KeyNotice::Unsupported => strings.unsupported.to_string(),
    }
}
