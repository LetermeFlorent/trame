mod column;
mod place;

use std::sync::Arc;

use crate::geometry::{Point, Rect};
use crate::locale::LanguageChoice;
use crate::text::TextLayout;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingControl {
    Close,
    RestoreText,
    TextSmaller,
    TextBigger,
    InterfaceSmaller,
    InterfaceBigger,
    Language(LanguageChoice),
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
pub struct LanguageOption {
    pub rect: Rect,
    pub choice: LanguageChoice,
    pub label: Arc<TextLayout>,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct SettingsFrame {
    pub backdrop: Rect,
    pub rect: Rect,
    pub title: SettingLine,
    pub close: Rect,
    pub sections: [SettingLine; 2],
    pub restore: SettingLine,
    pub toggle: Rect,
    pub restores_text: bool,
    pub text_size: Stepper,
    pub interface_size: Stepper,
    pub language: SettingLine,
    pub languages: Vec<LanguageOption>,
}

impl SettingsFrame {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let languages = self
            .languages
            .iter()
            .map(|option| (option.rect, SettingControl::Language(option.choice)));
        [
            (self.close, SettingControl::Close),
            (self.restore.rect, SettingControl::RestoreText),
            (self.text_size.smaller, SettingControl::TextSmaller),
            (self.text_size.bigger, SettingControl::TextBigger),
            (
                self.interface_size.smaller,
                SettingControl::InterfaceSmaller,
            ),
            (self.interface_size.bigger, SettingControl::InterfaceBigger),
        ]
        .into_iter()
        .chain(languages)
        .find(|(rect, _)| rect.contains(point))
        .map(|(_, control)| control)
    }
}
