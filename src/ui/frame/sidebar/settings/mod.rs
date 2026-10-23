mod column;
mod launch;
mod place;
mod popup;
mod rows;

use std::sync::Arc;

use crate::geometry::{Point, Rect};
use crate::locale::LanguageChoice;
use crate::preferences::Recall;
use crate::text::TextLayout;
use crate::ui::frame::FieldFrame;
pub(in crate::ui::frame) use column::{CAPTION_HEIGHT, LINE_HEIGHT, SECTION_GAP};
pub use popup::PopupFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingControl {
    Close,
    Recall(Recall),
    StartupCommand,
    ClearStartupCommand,
    InstanceTabsFewer,
    InstanceTabsMore,
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
pub struct Chip {
    pub rect: Rect,
    pub label: Arc<TextLayout>,
    pub control: SettingControl,
    pub selected: bool,
    pub faint: bool,
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
    pub popup: PopupFrame,
    pub sections: [SettingLine; 5],
    pub toggles: [ToggleRow; 3],
    pub command: ChoiceRow,
    pub command_field: Option<FieldFrame>,
    pub instance_tabs: Stepper,
    pub text_size: Stepper,
    pub interface_size: Stepper,
    pub language: ChoiceRow,
    pub folder: ChoiceRow,
    pub hosts: Vec<ChoiceRow>,
    pub hosts_hint: Option<SettingLine>,
}

impl SettingsFrame {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let field = self.command_field.as_ref().map(|field| field.rect);
        let toggles = self.toggles.iter().filter(|toggle| toggle.enabled);
        let rows = [&self.command, &self.language, &self.folder]
            .into_iter()
            .chain(&self.hosts)
            .flat_map(ChoiceRow::controls);
        field
            .map(|rect| (rect, SettingControl::StartupCommand))
            .into_iter()
            .chain([(self.popup.close, SettingControl::Close)])
            .chain(toggles.map(|toggle| (toggle.line.rect, toggle.control)))
            .chain([
                (
                    self.instance_tabs.smaller,
                    SettingControl::InstanceTabsFewer,
                ),
                (self.instance_tabs.bigger, SettingControl::InstanceTabsMore),
                (self.text_size.smaller, SettingControl::TextSmaller),
                (self.text_size.bigger, SettingControl::TextBigger),
                (
                    self.interface_size.smaller,
                    SettingControl::InterfaceSmaller,
                ),
                (self.interface_size.bigger, SettingControl::InterfaceBigger),
            ])
            .chain(rows)
            .find(|(rect, _)| rect.contains(point))
            .map(|(_, control)| control)
    }
}
