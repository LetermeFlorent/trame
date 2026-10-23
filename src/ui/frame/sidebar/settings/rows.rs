use crate::locale::{LanguageChoice, Strings};
use crate::preferences::Preferences;

use super::column::Column;
use super::{ChoiceRow, SettingControl, Stepper};

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
) -> ChoiceRow {
    let name = preferences.default_folder.as_deref().map(|path| {
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
    });
    picker_row(
        column,
        strings.default_folder,
        name.as_deref(),
        strings.choose,
        [SettingControl::DefaultFolder, SettingControl::ClearFolder],
    )
}

pub(super) fn host_rows(column: &mut Column, preferences: &Preferences) -> Vec<ChoiceRow> {
    let hosts = preferences.ssh_hosts.iter().enumerate();
    hosts
        .map(|(index, host)| {
            let forget = SettingControl::ForgetHost(index);
            column.choice_row(&host.label(), &[], Some(forget))
        })
        .collect()
}
