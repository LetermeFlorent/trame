mod controls;
mod height;
mod launch;
mod rows;

use std::path::Path;
use std::sync::Arc;

use crate::geometry::{Point, Rect};
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::text::TextLayout;
use crate::ui::frame::FieldFrame;

use super::column::Column;
use super::{ChoiceRow, SettingControl, SettingLine};
pub(super) use height::general_height;
use launch::{
    close_toggle, command_row, history_stepper, instance_tabs_stepper, recall_toggles,
    system_toggles,
};
use rows::{display_rows, folder_row, language_row, size_steppers};

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
}

#[derive(Clone, Debug)]
pub struct GeneralPage {
    pub sections: Vec<SettingLine>,
    pub toggles: [ToggleRow; 3],
    pub confirm_close: ToggleRow,
    pub system: Vec<ToggleRow>,
    pub command: ChoiceRow,
    pub command_field: Option<FieldFrame>,
    pub instance_tabs: Stepper,
    pub history: Stepper,
    pub text_size: Stepper,
    pub interface_size: Stepper,
    pub language: ChoiceRow,
    pub panel_side: ChoiceRow,
    pub density: ChoiceRow,
    pub folder: ChoiceRow,
}

impl GeneralPage {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let field = self.command_field.as_ref().map(|field| field.rect);
        let toggles = self.all_toggles().filter(|toggle| toggle.enabled);
        let rows = [
            &self.command,
            &self.language,
            &self.panel_side,
            &self.density,
            &self.folder,
        ]
        .into_iter()
        .flat_map(ChoiceRow::controls);
        field
            .map(|rect| (rect, SettingControl::StartupCommand))
            .into_iter()
            .chain(toggles.map(|toggle| (toggle.line.rect, toggle.control)))
            .chain(
                self.steppers()
                    .into_iter()
                    .flat_map(|(stepper, [shrink, grow])| {
                        [(stepper.smaller, shrink), (stepper.bigger, grow)]
                    }),
            )
            .chain(rows)
            .find(|(rect, _)| rect.contains(point))
            .map(|(_, control)| control)
    }

    pub fn all_toggles(&self) -> impl Iterator<Item = &ToggleRow> {
        self.toggles
            .iter()
            .chain([&self.confirm_close])
            .chain(&self.system)
    }

    pub fn steppers(&self) -> [(&Stepper, [SettingControl; 2]); 4] {
        use SettingControl::{
            HistoryFewer, HistoryMore, InstanceTabsFewer, InstanceTabsMore, InterfaceBigger,
            InterfaceSmaller, TextBigger, TextSmaller,
        };
        [
            (&self.instance_tabs, [InstanceTabsFewer, InstanceTabsMore]),
            (&self.history, [HistoryFewer, HistoryMore]),
            (&self.text_size, [TextSmaller, TextBigger]),
            (&self.interface_size, [InterfaceSmaller, InterfaceBigger]),
        ]
    }
}

pub(super) fn general_page(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
    folder: Option<&Path>,
) -> GeneralPage {
    let at_launch = column.caption(strings.at_launch);
    let toggles = recall_toggles(column, strings, preferences);
    let terminals = column.section(strings.terminals);
    let command = command_row(column, strings, preferences);
    let instance_tabs = instance_tabs_stepper(column, strings, preferences);
    let history = history_stepper(column, strings, preferences);
    let confirm_close = close_toggle(column, strings, preferences);
    let display = column.section(strings.display);
    let [text_size, interface_size] = size_steppers(column, strings, preferences);
    let language = language_row(column, strings, preferences);
    let [panel_side, density] = display_rows(column, strings, preferences);
    let folders = column.section(strings.folders);
    let folder = folder_row(column, strings, preferences, folder);
    let (system_section, system) = system_toggles(column, strings, preferences);
    let mut sections = vec![at_launch, terminals, display, folders];
    sections.extend(system_section);
    GeneralPage {
        sections,
        toggles,
        confirm_close,
        system,
        command,
        command_field: None,
        instance_tabs,
        history,
        text_size,
        interface_size,
        language,
        panel_side,
        density,
        folder,
    }
}
