use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::ui::frame::sidebar::settings::column::Column;

use super::{FormPage, SettingControl, ToggleRow};

const SUB_INDENT: f32 = 16.0;

pub(super) fn system_page(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> FormPage {
    let efficiency = preferences.efficiency;
    let mut toggles = vec![
        column.toggle_row(
            strings.efficiency_mode,
            [efficiency, true],
            SettingControl::Efficiency,
        ),
        sub_toggle(
            column,
            strings.rest_hidden_tabs,
            [preferences.rest_hidden_tabs, efficiency],
            SettingControl::RestHiddenTabs,
        ),
    ];
    let mut sections = Vec::new();
    if cfg!(windows) {
        let notify = [preferences.notify_waiting, true];
        let control = SettingControl::NotifyWaiting;
        toggles.push(column.toggle_row(strings.notify_waiting, notify, control));
        sections.push(column.section(strings.updates));
        let auto = [preferences.updates.auto, true];
        let control = SettingControl::AutoUpdate;
        toggles.push(column.toggle_row(strings.update_automatically, auto, control));
    }
    FormPage {
        sections,
        toggles,
        ..FormPage::default()
    }
}

fn sub_toggle(
    column: &mut Column,
    label: &str,
    state: [bool; 2],
    control: SettingControl,
) -> ToggleRow {
    let mut row = column.toggle_row(label, state, control);
    row.line.label_x += column.metrics.px(SUB_INDENT);
    row
}
