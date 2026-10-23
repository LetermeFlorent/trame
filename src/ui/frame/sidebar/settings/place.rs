use crate::geometry::Rect;
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::text::TextSystem;
use crate::ui::frame::FrameLayout;
use crate::ui::metrics::UiMetrics;

use super::column::{CAPTION_HEIGHT, Column, LINE_HEIGHT};
use super::{SettingsFrame, Stepper};

const PANEL_WIDTH: f32 = 360.0;
const PANEL_PAD: f32 = 12.0;
const TITLE_HEIGHT: f32 = 36.0;
const SECTION_GAP: f32 = 8.0;
const CAPTIONS: f32 = 2.0;
const LINES: f32 = 4.0;

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
        let rect = panel_rect(&metrics, backdrop);
        let pad = metrics.px(PANEL_PAD);
        let mut column = Column {
            text,
            metrics,
            inner: rect.inset(pad),
            y: rect.y + pad,
        };
        let title = column.line(strings.settings, TITLE_HEIGHT, metrics.strong());
        let close = column.end_button(&title);
        let tab_text = column.line(strings.tab_text, CAPTION_HEIGHT, metrics.caption());
        let restore = column.line(strings.restore_text, LINE_HEIGHT, metrics.label());
        let toggle = column.toggle(&restore);
        column.skip(SECTION_GAP);
        let display = column.line(strings.display, CAPTION_HEIGHT, metrics.caption());
        let [text_size, interface_size] = size_steppers(&mut column, strings, preferences);
        let language = column.line(strings.language, LINE_HEIGHT, metrics.label());
        let languages = column.choices(&language, preferences.language);
        self.settings = Some(SettingsFrame {
            backdrop,
            rect,
            title,
            close,
            sections: [tab_text, display],
            restore,
            toggle,
            restores_text: preferences.restore_text,
            text_size,
            interface_size,
            language,
            languages,
        });
    }
}

fn size_steppers(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> [Stepper; 2] {
    let can_step = |step: fn(Preferences, i32) -> Preferences| {
        [-1, 1].map(|steps| step(*preferences, steps) != *preferences)
    };
    [
        column.stepper(
            strings.text_size,
            &format!("{:.0}", preferences.font_size),
            can_step(Preferences::step_font_size),
        ),
        column.stepper(
            strings.interface_size,
            &format!("{}%", preferences.interface_size),
            can_step(Preferences::step_interface_size),
        ),
    ]
}

fn panel_rect(metrics: &UiMetrics, backdrop: Rect) -> Rect {
    let width = metrics.px(PANEL_WIDTH).min(backdrop.width);
    let height = 2.0 * metrics.px(PANEL_PAD)
        + metrics.px(TITLE_HEIGHT)
        + CAPTIONS * metrics.px(CAPTION_HEIGHT)
        + LINES * metrics.px(LINE_HEIGHT)
        + metrics.px(SECTION_GAP);
    let x = backdrop.x + (backdrop.width - width) / 2.0;
    let y = (backdrop.y + (backdrop.height - height) / 2.0).max(backdrop.y);
    Rect::new(x, y, width, height).snapped()
}
