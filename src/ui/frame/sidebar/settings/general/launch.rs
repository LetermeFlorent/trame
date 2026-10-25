use crate::locale::Strings;
use crate::preferences::{Preferences, Recall};

use super::rows::{can_step, picker_row};
use super::{ChoiceRow, SettingControl, SettingLine, Stepper, ToggleRow};
use crate::ui::frame::sidebar::settings::column::Column;

pub(super) fn recall_toggles(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> [ToggleRow; 3] {
    Recall::ALL.map(|recall| {
        let label = match recall {
            Recall::Instances => strings.remember_instances,
            Recall::Tabs => strings.remember_tabs,
            Recall::Text => strings.restore_text,
        };
        let state = [preferences.recalls(recall), preferences.can_recall(recall)];
        column.toggle_row(label, state, SettingControl::Recall(recall))
    })
}

pub(super) fn close_toggle(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> ToggleRow {
    let state = [preferences.confirm_close, true];
    column.toggle_row(strings.confirm_close, state, SettingControl::ConfirmClose)
}

pub(super) fn system_toggles(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> (Option<SettingLine>, Vec<ToggleRow>) {
    if !cfg!(windows) {
        return (None, Vec::new());
    }
    let section = column.section(strings.windows_section);
    let toggles = [
        (
            strings.efficiency_mode,
            preferences.efficiency,
            SettingControl::Efficiency,
        ),
        (
            strings.acrylic_panel,
            preferences.acrylic,
            SettingControl::Acrylic,
        ),
        (
            strings.notify_waiting,
            preferences.notify_waiting,
            SettingControl::NotifyWaiting,
        ),
    ]
    .map(|(label, on, control)| column.toggle_row(label, [on, true], control));
    (Some(section), toggles.into())
}

pub(super) fn command_row(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> ChoiceRow {
    picker_row(
        column,
        strings.startup_command,
        preferences.startup_command.as_deref(),
        strings.add,
        [
            SettingControl::StartupCommand,
            SettingControl::ClearStartupCommand,
        ],
    )
}

pub(super) fn instance_tabs_stepper(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> Stepper {
    column.stepper(
        strings.instance_tabs,
        &preferences.instance_tabs.to_string(),
        can_step(preferences, Preferences::step_instance_tabs),
    )
}
