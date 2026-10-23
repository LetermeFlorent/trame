use crate::geometry::Rect;
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::text::TextSystem;
use crate::ui::frame::FrameLayout;
use crate::ui::metrics::UiMetrics;

use super::SettingsFrame;
use super::column::{CAPTION_HEIGHT, Column, LINE_HEIGHT};
use super::rows::{folder_row, host_rows, language_row, size_steppers};

const PANEL_WIDTH: f32 = 360.0;
const PANEL_PAD: f32 = 12.0;
const TITLE_HEIGHT: f32 = 36.0;
const SECTION_GAP: f32 = 8.0;
const CAPTIONS: f32 = 4.0;
const FIXED_LINES: usize = 5;

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
    let tab_text = column.line(strings.tab_text, CAPTION_HEIGHT, metrics.caption());
    let restore = column.line(strings.restore_text, LINE_HEIGHT, metrics.label());
    let toggle = column.toggle(&restore);
    column.skip(SECTION_GAP);
    let display = column.line(strings.display, CAPTION_HEIGHT, metrics.caption());
    let [text_size, interface_size] = size_steppers(column, strings, preferences);
    let language = language_row(column, strings, preferences);
    column.skip(SECTION_GAP);
    let folders = column.line(strings.folders, CAPTION_HEIGHT, metrics.caption());
    let folder = folder_row(column, strings, preferences);
    column.skip(SECTION_GAP);
    let connections = column.line(strings.connections, CAPTION_HEIGHT, metrics.caption());
    let hosts = host_rows(column, preferences);
    let hint = hosts
        .is_empty()
        .then(|| column.line(strings.no_connections, LINE_HEIGHT, metrics.label()));
    SettingsFrame {
        backdrop,
        rect,
        title,
        close,
        sections: [tab_text, display, folders, connections],
        restore,
        toggle,
        restores_text: preferences.restore_text,
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
    let x = backdrop.x + (backdrop.width - width) / 2.0;
    let y = (backdrop.y + (backdrop.height - height) / 2.0).max(backdrop.y);
    Rect::new(x, y, width, height).snapped()
}
