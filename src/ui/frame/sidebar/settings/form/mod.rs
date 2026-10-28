mod controls;
mod launch;
mod lookup;
mod rows;
mod system;

use std::path::Path;
use std::sync::Arc;

use crate::geometry::Rect;
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::text::TextLayout;
use crate::ui::frame::FieldFrame;
use crate::ui::metrics::UiMetrics;

use super::column::Column;
use super::{ChoiceRow, SettingControl, SettingLine, SettingsTab};
use launch::{
    command_row, history_stepper, instance_tabs_stepper, question_toggles, recall_toggles,
};
use rows::{display_rows, folder_row, language_row, size_steppers};
use system::system_page;

const PAGE_HEIGHT: f32 = 640.0;

#[derive(Clone, Debug)]
pub struct ToggleRow {
    pub line: SettingLine,
    pub switch: Rect,
    pub on: bool,
    pub enabled: bool,
    pub control: SettingControl,
}

#[derive(Clone, Debug)]
pub struct Stepper {
    pub line: SettingLine,
    pub smaller: Rect,
    pub bigger: Rect,
    pub value: Arc<TextLayout>,
    pub value_rect: Rect,
    pub can_shrink: bool,
    pub can_grow: bool,
    pub controls: [SettingControl; 2],
    pub typed: Option<SettingControl>,
}

#[derive(Clone, Debug, Default)]
pub struct FormPage {
    pub sections: Vec<SettingLine>,
    pub toggles: Vec<ToggleRow>,
    pub steppers: Vec<Stepper>,
    pub rows: Vec<ChoiceRow>,
    pub field: Option<(SettingControl, FieldFrame)>,
}

pub(super) fn page_height(metrics: &UiMetrics, room: f32) -> f32 {
    metrics.px(PAGE_HEIGHT).min(room).max(0.0)
}

pub(super) fn form_page(
    column: &mut Column,
    tab: SettingsTab,
    strings: &Strings,
    preferences: &Preferences,
    folder: Option<&Path>,
) -> FormPage {
    let mut page = FormPage::default();
    match tab {
        SettingsTab::Terminals => {
            page.rows.push(command_row(column, strings, preferences));
            page.steppers = vec![
                instance_tabs_stepper(column, strings, preferences),
                history_stepper(column, strings, preferences),
            ];
            page.toggles = question_toggles(column, strings, preferences).into();
        }
        SettingsTab::Display => {
            page.steppers = size_steppers(column, strings, preferences).into();
            page.rows.push(language_row(column, strings, preferences));
            page.rows.extend(display_rows(column, strings, preferences));
        }
        SettingsTab::System => return system_page(column, strings, preferences),
        _ => {
            page.sections.push(column.caption(strings.at_launch));
            page.toggles = recall_toggles(column, strings, preferences).into();
            page.sections.push(column.section(strings.folders));
            page.rows
                .push(folder_row(column, strings, preferences, folder));
        }
    }
    page
}
