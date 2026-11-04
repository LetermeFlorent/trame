use crate::locale::Strings;
use crate::preferences::{Preferences, Recall};

use super::rows::{can_step, picker_row};
use super::{ChoiceRow, SettingControl, Stepper, ToggleRow};
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

pub(super) fn question_toggles(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> [ToggleRow; 2] {
    let close = [preferences.confirm_close, true];
    let paste = [preferences.confirm_paste, true];
    [
        column.toggle_row(strings.confirm_close, close, SettingControl::ConfirmClose),
        column.toggle_row(strings.confirm_paste, paste, SettingControl::ConfirmPaste),
    ]
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
        [
            SettingControl::InstanceTabsFewer,
            SettingControl::InstanceTabsMore,
        ],
    )
}

pub(super) fn history_stepper(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> Stepper {
    let stepper = column.stepper(
        strings.history_lines,
        &preferences.history_lines.to_string(),
        can_step(preferences, Preferences::step_history_lines),
        [SettingControl::HistoryFewer, SettingControl::HistoryMore],
    );
    Stepper {
        typed: Some(SettingControl::HistoryLines),
        ..stepper
    }
}
