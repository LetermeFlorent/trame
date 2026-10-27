use std::path::Path;

use crate::geometry::Point;
use crate::locale::Strings;
use crate::preferences::Preferences;

use super::{
    Chip, ConnectionsPage, GeneralPage, HostsView, InstancesPage, PopupFrame, SettingControl,
    SettingsTab, ShortcutsPage, ShortcutsView,
};

#[derive(Clone, Copy)]
pub struct SettingsRequest<'a> {
    pub preferences: &'a Preferences,
    pub strings: &'static Strings,
    pub folder: Option<&'a Path>,
    pub tab: SettingsTab,
    pub hosts: Option<HostsView<'a>>,
    pub shortcuts: Option<ShortcutsView>,
}

#[derive(Clone, Debug)]
pub enum SettingsPage {
    General(Box<GeneralPage>),
    Instances(Box<InstancesPage>),
    Connections(Box<ConnectionsPage>),
    Shortcuts(Box<ShortcutsPage>),
}

#[derive(Clone, Debug)]
pub struct SettingsFrame {
    pub popup: PopupFrame,
    pub tabs: [Chip; 4],
    pub page: SettingsPage,
}

impl SettingsFrame {
    pub fn general(&self) -> Option<&GeneralPage> {
        match &self.page {
            SettingsPage::General(page) => Some(page),
            _ => None,
        }
    }

    pub fn connections(&self) -> Option<&ConnectionsPage> {
        match &self.page {
            SettingsPage::Connections(page) => Some(page),
            _ => None,
        }
    }

    pub fn shortcuts(&self) -> Option<&ShortcutsPage> {
        match &self.page {
            SettingsPage::Shortcuts(page) => Some(page),
            _ => None,
        }
    }

    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let page = match &self.page {
            SettingsPage::General(page) => page.control_at(point),
            SettingsPage::Instances(page) => page.control_at(point),
            SettingsPage::Connections(page) => page.control_at(point),
            SettingsPage::Shortcuts(page) => page.control_at(point),
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
