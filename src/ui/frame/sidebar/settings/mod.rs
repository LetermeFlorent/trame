mod column;

use std::sync::Arc;

use crate::geometry::{Point, Rect};
use crate::preferences::Preferences;
use crate::text::{TextLayout, TextSystem};
use crate::ui::frame::FrameLayout;
use column::{CAPTION_HEIGHT, Column, LINE_HEIGHT};

const CAPTION: &str = "SETTINGS";
const RESTORE_LABEL: &str = "Restore tab text";
const TEXT_LABEL: &str = "Text size";
const INTERFACE_LABEL: &str = "Interface size";
const PANEL_WIDTH: f32 = 248.0;
const PANEL_PAD: f32 = 8.0;
const PANEL_GAP: f32 = 6.0;
const LINES: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingControl {
    RestoreText,
    TextSmaller,
    TextBigger,
    InterfaceSmaller,
    InterfaceBigger,
}

#[derive(Clone, Debug)]
pub struct SettingLine {
    pub rect: Rect,
    pub label: Arc<TextLayout>,
    pub label_x: f32,
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
pub struct SettingsFrame {
    pub rect: Rect,
    pub caption: SettingLine,
    pub restore: SettingLine,
    pub toggle: Rect,
    pub restores_text: bool,
    pub text_size: Stepper,
    pub interface_size: Stepper,
}

impl SettingsFrame {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let controls = [
            (self.restore.rect, SettingControl::RestoreText),
            (self.text_size.smaller, SettingControl::TextSmaller),
            (self.text_size.bigger, SettingControl::TextBigger),
            (
                self.interface_size.smaller,
                SettingControl::InterfaceSmaller,
            ),
            (self.interface_size.bigger, SettingControl::InterfaceBigger),
        ];
        controls
            .iter()
            .find(|(rect, _)| rect.contains(point))
            .map(|(_, control)| *control)
    }
}

impl FrameLayout {
    pub fn place_settings(&mut self, text: &mut TextSystem, preferences: &Preferences) {
        let metrics = self.metrics;
        let pad = metrics.px(PANEL_PAD);
        let width = metrics.px(PANEL_WIDTH);
        let height = pad * 2.0 + metrics.px(CAPTION_HEIGHT) + metrics.px(LINE_HEIGHT) * LINES;
        let anchor = self.footer.settings;
        let top = (anchor.y - metrics.px(PANEL_GAP) - height).max(self.bounds.y);
        let rect = Rect::new(anchor.x, top, width, height);
        let mut column = Column {
            text,
            metrics,
            inner: rect.inset(pad),
            y: rect.y + pad,
        };
        let caption = column.line(CAPTION, CAPTION_HEIGHT, metrics.caption());
        let restore = column.line(RESTORE_LABEL, LINE_HEIGHT, metrics.label());
        let toggle = column.toggle(&restore);
        let moves = |stepped: Preferences| stepped != *preferences;
        let text_size = column.stepper(
            TEXT_LABEL,
            &format!("{:.0}", preferences.font_size),
            [-1, 1].map(|steps| moves(preferences.step_font_size(steps))),
        );
        let interface_size = column.stepper(
            INTERFACE_LABEL,
            &format!("{}%", preferences.interface_size),
            [-1, 1].map(|steps| moves(preferences.step_interface_size(steps))),
        );
        self.settings = Some(SettingsFrame {
            rect,
            caption,
            restore,
            toggle,
            restores_text: preferences.restore_text,
            text_size,
            interface_size,
        });
    }
}
