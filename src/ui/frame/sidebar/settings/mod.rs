mod chips;
mod column;
mod form;
mod hosts;
mod instances;
mod message;
mod page;
mod place;
mod popup;
mod shortcuts;

use std::sync::Arc;

use crate::geometry::Rect;
use crate::locale::LanguageChoice;
use crate::preferences::Recall;
use crate::text::TextLayout;
use crate::ui::display::{Density, PanelSide};
pub(in crate::ui::frame) use column::{
    CAPTION_HEIGHT, Column, LINE_HEIGHT, LINE_INSET, SECTION_GAP,
};
pub use form::{FormPage, Stepper, ToggleRow};
pub use hosts::{ConnectionsPage, HostList, HostRow, HostsView, ListControl, ScrollBar};
pub(in crate::ui::frame) use hosts::{ListSpec, host_labels};
pub use instances::{InstancesPage, NumberChoice};
pub use message::{MessageFrame, MessageRequest};
pub use page::{SettingsFrame, SettingsPage, SettingsRequest};
pub use popup::PopupFrame;
pub use shortcuts::{
    KeyControl, KeyDetail, KeyLine, KeyNotice, ShortcutsPage, ShortcutsView, key_label,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsTab {
    #[default]
    General,
    Terminals,
    Display,
    System,
    Instances,
    Connections,
    Shortcuts,
}

impl SettingsTab {
    pub const ALL: [Self; 7] = [
        Self::General,
        Self::Terminals,
        Self::Display,
        Self::System,
        Self::Instances,
        Self::Connections,
        Self::Shortcuts,
    ];
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
    HistoryLines,
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
    RestHiddenTabs,
    NotifyWaiting,
    UpdateAtLaunch,
    UpdateAhead,
    Numbers(NumberChoice),
    Keys(KeyControl),
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
