mod column;
mod place;
mod rows;

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
    DefaultFolder,
    ClearFolder,
    ForgetHost(usize),
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
pub struct Chip {
    pub rect: Rect,
    pub label: Arc<TextLayout>,
    pub control: SettingControl,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct ChoiceRow {
    pub line: SettingLine,
    pub chips: Vec<Chip>,
    pub clear: Option<(Rect, SettingControl)>,
}

impl ChoiceRow {
    pub fn controls(&self) -> impl Iterator<Item = (Rect, SettingControl)> + '_ {
        self.chips
            .iter()
            .map(|chip| (chip.rect, chip.control))
            .chain(self.clear)
    }
}

#[derive(Clone, Debug)]
pub struct SettingsFrame {
    pub backdrop: Rect,
    pub rect: Rect,
    pub title: SettingLine,
    pub close: Rect,
    pub sections: [SettingLine; 4],
    pub restore: SettingLine,
    pub toggle: Rect,
    pub restores_text: bool,
    pub text_size: Stepper,
    pub interface_size: Stepper,
    pub language: ChoiceRow,
    pub folder: ChoiceRow,
    pub hosts: Vec<ChoiceRow>,
    pub hosts_hint: Option<SettingLine>,
}

impl SettingsFrame {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let rows = [&self.language, &self.folder]
            .into_iter()
            .chain(&self.hosts)
            .flat_map(ChoiceRow::controls);
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
        .chain(rows)
        .find(|(rect, _)| rect.contains(point))
        .map(|(_, control)| control)
    }
}
