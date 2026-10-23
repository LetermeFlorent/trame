use crate::geometry::Rect;
use crate::input::TextField;
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::text::TextSystem;
use crate::ui::frame::FrameLayout;
use crate::ui::frame::editor::place_field;
use crate::ui::metrics::UiMetrics;

use super::SettingsFrame;
use super::column::{CAPTION_HEIGHT, Column, LINE_HEIGHT, SECTION_GAP};
use super::launch::{command_row, instance_tabs_stepper, recall_toggles};
use super::rows::{folder_row, host_rows, language_row, size_steppers};

const PANEL_WIDTH: f32 = 360.0;
const PANEL_PAD: f32 = 12.0;
const TITLE_HEIGHT: f32 = 36.0;
const CAPTIONS: f32 = 5.0;
const FIXED_LINES: usize = 9;

impl FrameLayout {
    pub fn place_settings(
        &mut self,
        text: &mut TextSystem,
        preferences: &Preferences,
        strings: &Strings,
    ) {
        let metrics = self.metrics;
        let bounds = self.bounds;
        let backdrop = Rect::from_edges(
            bounds.x,
            self.title_bar.rect.bottom(),
            bounds.right(),
            bounds.bottom(),
        );
        let lines = FIXED_LINES + preferences.ssh_hosts.len().max(1);
        let rect = panel_rect(&metrics, backdrop, lines);
        let pad = metrics.px(PANEL_PAD);
        let mut column = Column {
            text,
            metrics,
            inner: rect.inset(pad),
            y: rect.y + pad,
        };
        self.settings = Some(settings_frame(
            &mut column,
            strings,
            preferences,
            [backdrop, rect],
        ));
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
    [backdrop, rect]: [Rect; 2],
) -> SettingsFrame {
    let metrics = column.metrics;
    let title = column.line(strings.settings, TITLE_HEIGHT, metrics.strong());
    let close = column.end_button(&title);
    let at_launch = column.caption(strings.at_launch);
    let toggles = recall_toggles(column, strings, preferences);
    let terminals = column.section(strings.terminals);
    let command = command_row(column, strings, preferences);
    let instance_tabs = instance_tabs_stepper(column, strings, preferences);
    let display = column.section(strings.display);
    let [text_size, interface_size] = size_steppers(column, strings, preferences);
    let language = language_row(column, strings, preferences);
    let folders = column.section(strings.folders);
    let folder = folder_row(column, strings, preferences);
    let connections = column.section(strings.connections);
    let hosts = host_rows(column, preferences);
    let hint = hosts
        .is_empty()
        .then(|| column.line(strings.no_connections, LINE_HEIGHT, metrics.label()));
    SettingsFrame {
        backdrop,
        rect,
        title,
        close,
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

fn panel_rect(metrics: &UiMetrics, backdrop: Rect, lines: usize) -> Rect {
    let width = metrics.px(PANEL_WIDTH).min(backdrop.width);
    let height = 2.0 * metrics.px(PANEL_PAD)
        + metrics.px(TITLE_HEIGHT)
        + CAPTIONS * metrics.px(CAPTION_HEIGHT)
        + lines as f32 * metrics.px(LINE_HEIGHT)
        + (CAPTIONS - 1.0) * metrics.px(SECTION_GAP);
    let x = backdrop.centered_x(width);
    let y = backdrop.centered_y(height).max(backdrop.y);
    Rect::new(x, y, width, height).snapped()
}
