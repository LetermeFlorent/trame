use std::path::Path;

use crate::geometry::Rect;
use crate::input::TextField;
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::text::TextSystem;
use crate::ui::frame::FrameLayout;
use crate::ui::frame::editor::place_field;
use crate::ui::metrics::UiMetrics;

use super::column::{CAPTION_HEIGHT, Column, LINE_HEIGHT, SECTION_GAP};
use super::launch::{command_row, instance_tabs_stepper, recall_toggles};
use super::rows::{folder_row, host_rows, language_row, size_steppers};
use super::{PopupFrame, SettingsFrame};

const CAPTIONS: f32 = 5.0;
const FIXED_LINES: usize = 9;

impl FrameLayout {
    pub fn place_settings(
        &mut self,
        text: &mut TextSystem,
        preferences: &Preferences,
        strings: &Strings,
        folder: Option<&Path>,
    ) {
        let lines = FIXED_LINES + preferences.ssh_hosts.len().max(1);
        let content = content_height(&self.metrics, lines);
        let (popup, mut column) = self.open_popup(text, strings.settings, content);
        let frame = settings_frame(&mut column, strings, preferences, popup, folder);
        self.settings = Some(frame);
    }

    pub fn place_command_field(&mut self, text: &mut TextSystem, field: &TextField) {
        let metrics = self.metrics;
        let Some(settings) = &mut self.settings else {
            return;
        };
        let line = settings.command.line.rect;
        let anchor = Rect::from_edges(line.center().x, line.y, line.right(), line.bottom());
        settings.command_field = Some(place_field(text, &metrics, anchor, line, field));
    }
}

fn settings_frame(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
    popup: PopupFrame,
    folder: Option<&Path>,
) -> SettingsFrame {
    let metrics = column.metrics;
    let at_launch = column.caption(strings.at_launch);
    let toggles = recall_toggles(column, strings, preferences);
    let terminals = column.section(strings.terminals);
    let command = command_row(column, strings, preferences);
    let instance_tabs = instance_tabs_stepper(column, strings, preferences);
    let display = column.section(strings.display);
    let [text_size, interface_size] = size_steppers(column, strings, preferences);
    let language = language_row(column, strings, preferences);
    let folders = column.section(strings.folders);
    let folder = folder_row(column, strings, preferences, folder);
    let connections = column.section(strings.connections);
    let hosts = host_rows(column, preferences);
    let hint = hosts
        .is_empty()
        .then(|| column.line(strings.no_connections, LINE_HEIGHT, metrics.label()));
    SettingsFrame {
        popup,
        sections: [at_launch, terminals, display, folders, connections],
        toggles,
        command,
        command_field: None,
        instance_tabs,
        text_size,
        interface_size,
        language,
        folder,
        hosts,
        hosts_hint: hint,
    }
}

fn content_height(metrics: &UiMetrics, lines: usize) -> f32 {
    CAPTIONS * metrics.px(CAPTION_HEIGHT)
        + lines as f32 * metrics.px(LINE_HEIGHT)
        + (CAPTIONS - 1.0) * metrics.px(SECTION_GAP)
}
