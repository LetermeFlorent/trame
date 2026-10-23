mod chips;
mod column;
mod general;
mod hosts;
mod place;
mod popup;

use std::path::Path;
use std::sync::Arc;

use crate::geometry::{Point, Rect};
use crate::locale::{LanguageChoice, Strings};
use crate::preferences::{Preferences, Recall};
use crate::text::TextLayout;
pub(in crate::ui::frame) use column::{CAPTION_HEIGHT, LINE_HEIGHT, LINE_INSET, SECTION_GAP};
pub use general::{GeneralPage, Stepper, ToggleRow};
pub(in crate::ui::frame) use hosts::ListSpec;
pub use hosts::{ConnectionsPage, HostList, HostRow, HostsView, ListControl, ScrollBar};
pub use popup::PopupFrame;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsTab {
    #[default]
    General,
    Connections,
}

impl SettingsTab {
    pub const ALL: [Self; 2] = [Self::General, Self::Connections];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingControl {
    Close,
    Tab(SettingsTab),
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
    AddHost,
}

#[derive(Clone, Debug)]
pub struct SettingLine {
    pub rect: Rect,
    pub label: Arc<TextLayout>,
    pub label_x: f32,
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

#[derive(Clone, Copy)]
pub struct SettingsRequest<'a> {
    pub preferences: &'a Preferences,
    pub strings: &'static Strings,
    pub folder: Option<&'a Path>,
    pub tab: SettingsTab,
    pub hosts: Option<HostsView<'a>>,
}

#[derive(Clone, Debug)]
pub enum SettingsPage {
    General(Box<GeneralPage>),
    Connections(Box<ConnectionsPage>),
}

#[derive(Clone, Debug)]
pub struct SettingsFrame {
    pub popup: PopupFrame,
    pub tabs: [Chip; 2],
    pub page: SettingsPage,
}

impl SettingsFrame {
    pub fn general(&self) -> Option<&GeneralPage> {
        match &self.page {
            SettingsPage::General(page) => Some(page),
            SettingsPage::Connections(_) => None,
        }
    }

    pub fn connections(&self) -> Option<&ConnectionsPage> {
        match &self.page {
            SettingsPage::Connections(page) => Some(page),
            SettingsPage::General(_) => None,
        }
    }

    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let page = match &self.page {
            SettingsPage::General(page) => page.control_at(point),
            SettingsPage::Connections(page) => page.control_at(point),
        };
        let tabs = self.tabs.iter().map(|chip| (chip.rect, chip.control));
        [(self.popup.close, SettingControl::Close)]
            .into_iter()
            .chain(tabs)
            .find(|(rect, _)| rect.contains(point))
            .map(|(_, control)| control)
            .or(page)
    }
}
