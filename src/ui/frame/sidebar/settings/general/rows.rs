use std::path::Path;

use crate::files::{home_dir, short_path};
use crate::locale::{LanguageChoice, Strings};
use crate::preferences::Preferences;
use crate::ui::frame::fit_text_start;

use super::{ChoiceRow, SettingControl, Stepper};
use crate::ui::frame::sidebar::settings::column::Column;

pub(super) fn size_steppers(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> [Stepper; 2] {
    [
        column.stepper(
            strings.text_size,
            &format!("{:.0}", preferences.font_size),
            can_step(preferences, Preferences::step_font_size),
        ),
        column.stepper(
            strings.interface_size,
            &format!("{}%", preferences.interface_size),
            can_step(preferences, Preferences::step_interface_size),
        ),
    ]
}

pub(super) fn can_step(
    preferences: &Preferences,
    step: fn(&Preferences, i32) -> Preferences,
) -> [bool; 2] {
    [-1, 1].map(|steps| step(preferences, steps) != *preferences)
}

pub(super) fn picker_row(
    column: &mut Column,
    label: &str,
    value: Option<&str>,
    empty: &str,
    [pick, clear]: [SettingControl; 2],
) -> ChoiceRow {
    let chip = (value.unwrap_or(empty), pick, false);
    column.choice_row(label, &[chip], value.map(|_| clear))
}

pub(super) fn language_row(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> ChoiceRow {
    let choices = LanguageChoice::ALL.map(|choice| {
        let selected = choice == preferences.language;
        (choice.label(), SettingControl::Language(choice), selected)
    });
    column.choice_row(strings.language, &choices, None)
}

pub(super) fn folder_row(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
    current: Option<&Path>,
) -> ChoiceRow {
    let chosen = preferences.default_folder.as_deref();
    let shown = chosen.or(current).map(|path| {
        let short = short_path(path, home_dir().as_deref());
        let room = column.chip_room();
        fit_text_start(column.text, &short, column.metrics.label(), room)
    });
    let pick = shown.as_deref().unwrap_or(strings.choose);
    let clear = chosen.map(|_| SettingControl::ClearFolder);
    let chips = [(pick, SettingControl::DefaultFolder, false)];
    let mut row = column.choice_row(strings.default_folder, &chips, clear);
    let suggested = chosen.is_none() && shown.is_some();
    for chip in &mut row.chips {
        chip.faint = suggested;
    }
    row
}
