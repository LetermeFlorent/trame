mod controls;
mod launch;
mod rows;

use std::path::Path;
use std::sync::Arc;

use crate::geometry::{Point, Rect};
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::text::TextLayout;
use crate::ui::frame::FieldFrame;
use crate::ui::metrics::UiMetrics;

use super::column::{CAPTION_HEIGHT, Column, LINE_HEIGHT, SECTION_GAP};
use super::{ChoiceRow, SettingControl, SettingLine};
use launch::{close_toggle, command_row, instance_tabs_stepper, recall_toggles};
use rows::{folder_row, language_row, size_steppers};

const CAPTIONS: f32 = 4.0;
const LINES: f32 = 10.0;

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
    pub sections: [SettingLine; 4],
    pub toggles: [ToggleRow; 3],
    pub confirm_close: ToggleRow,
    pub command: ChoiceRow,
    pub command_field: Option<FieldFrame>,
    pub instance_tabs: Stepper,
    pub text_size: Stepper,
    pub interface_size: Stepper,
    pub language: ChoiceRow,
    pub folder: ChoiceRow,
}

impl GeneralPage {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let field = self.command_field.as_ref().map(|field| field.rect);
        let toggles = self.all_toggles().filter(|toggle| toggle.enabled);
        let rows = [&self.command, &self.language, &self.folder]
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
        self.toggles.iter().chain([&self.confirm_close])
    }

    pub fn steppers(&self) -> [(&Stepper, [SettingControl; 2]); 3] {
        use SettingControl::{
            InstanceTabsFewer, InstanceTabsMore, InterfaceBigger, InterfaceSmaller, TextBigger,
            TextSmaller,
        };
        [
            (&self.instance_tabs, [InstanceTabsFewer, InstanceTabsMore]),
            (&self.text_size, [TextSmaller, TextBigger]),
            (&self.interface_size, [InterfaceSmaller, InterfaceBigger]),
        ]
    }
}

pub(super) fn general_height(metrics: &UiMetrics) -> f32 {
    CAPTIONS * metrics.px(CAPTION_HEIGHT)
        + LINES * metrics.px(LINE_HEIGHT)
        + (CAPTIONS - 1.0) * metrics.px(SECTION_GAP)
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
    let confirm_close = close_toggle(column, strings, preferences);
    let display = column.section(strings.display);
    let [text_size, interface_size] = size_steppers(column, strings, preferences);
    let language = language_row(column, strings, preferences);
    let folders = column.section(strings.folders);
    let folder = folder_row(column, strings, preferences, folder);
    GeneralPage {
        sections: [at_launch, terminals, display, folders],
        toggles,
        confirm_close,
        command,
        command_field: None,
        instance_tabs,
        text_size,
        interface_size,
        language,
        folder,
    }
}
