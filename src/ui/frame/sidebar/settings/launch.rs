use crate::locale::Strings;
use crate::preferences::{Preferences, Recall};

use super::column::{Column, LINE_HEIGHT};
use super::rows::{can_step, picker_row};
use super::{ChoiceRow, SettingControl, Stepper, ToggleRow};

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
        let line = column.line(label, LINE_HEIGHT, column.metrics.label());
        ToggleRow {
            switch: column.toggle(&line),
            on: preferences.recalls(recall),
            enabled: preferences.can_recall(recall),
            control: SettingControl::Recall(recall),
            line,
        }
    })
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
