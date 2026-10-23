use crate::locale::{LanguageChoice, Strings};
use crate::preferences::Preferences;

use super::column::Column;
use super::{ChoiceRow, SettingControl, Stepper};

pub(super) fn size_steppers(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> [Stepper; 2] {
    let can_step = |step: fn(&Preferences, i32) -> Preferences| {
        [-1, 1].map(|steps| step(preferences, steps) != *preferences)
    };
    [
        column.stepper(
            strings.text_size,
            &format!("{:.0}", preferences.font_size),
            can_step(Preferences::step_font_size),
        ),
        column.stepper(
            strings.interface_size,
            &format!("{}%", preferences.interface_size),
            can_step(Preferences::step_interface_size),
        ),
    ]
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
    let folder = preferences.default_folder.as_deref();
    let name = folder.map(|path| {
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
    });
    let chip = (
        name.as_deref().unwrap_or(strings.choose),
        SettingControl::DefaultFolder,
        false,
    );
    let clear = folder.map(|_| SettingControl::ClearFolder);
    column.choice_row(strings.default_folder, &[chip], clear)
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
