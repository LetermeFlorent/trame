mod chips;
mod column;
mod general;
mod hosts;
mod instances;
mod message;
mod page;
mod place;
mod popup;

use std::sync::Arc;

use crate::geometry::Rect;
use crate::locale::LanguageChoice;
use crate::preferences::Recall;
use crate::text::TextLayout;
use crate::ui::display::{Density, PanelSide};
pub(in crate::ui::frame) use column::{
    CAPTION_HEIGHT, Column, LINE_HEIGHT, LINE_INSET, SECTION_GAP,
};
pub use general::{GeneralPage, Stepper, ToggleRow};
pub use hosts::{ConnectionsPage, HostList, HostRow, HostsView, ListControl, ScrollBar};
pub(in crate::ui::frame) use hosts::{ListSpec, host_labels};
pub use instances::{InstancesPage, NumberChoice};
pub use message::{MessageFrame, MessageRequest};
pub use page::{SettingsFrame, SettingsPage, SettingsRequest};
pub use popup::PopupFrame;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsTab {
    #[default]
    General,
    Instances,
    Connections,
}

impl SettingsTab {
    pub const ALL: [Self; 3] = [Self::General, Self::Instances, Self::Connections];
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
    HistoryFewer,
    HistoryMore,
    TextSmaller,
    TextBigger,
    InterfaceSmaller,
    InterfaceBigger,
    Language(LanguageChoice),
    PanelSide(PanelSide),
    Density(Density),
    DefaultFolder,
    ClearFolder,
    AddHost,
    ConfirmClose,
    Efficiency,
    NotifyWaiting,
    Numbers(NumberChoice),
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
