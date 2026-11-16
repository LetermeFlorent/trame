use std::path::PathBuf;

use crate::desk::{Desk, Effects};
use crate::session::SessionId;
use crate::workspace::{EditTarget, Instance, InstanceId};

use super::Placement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolderTarget {
    Default,
    Instance(InstanceId),
    Tab(SessionId),
}

impl From<EditTarget> for FolderTarget {
    fn from(target: EditTarget) -> Self {
        match target {
            EditTarget::Instance(id) => Self::Instance(id),
            EditTarget::Tab(id) => Self::Tab(id),
        }
    }
}

impl Desk {
    pub fn folder_picked(&mut self, target: FolderTarget, folder: PathBuf) -> Effects {
        match target {
            FolderTarget::Default => {
                let next = self
                    .preferences
                    .with(|next| next.default_folder = Some(folder));
                self.store_preferences(next)
            }
            FolderTarget::Instance(id) => {
                self.workspace.set_instance_folder(id, Some(folder.clone()));
                let tabs = self.workspace.instance(id).map(Instance::tabs);
                self.relaunch(tabs.unwrap_or_default(), folder)
            }
            FolderTarget::Tab(id) => self.relaunch(vec![id], folder),
        }
    }

    pub fn start_folder(&self, placement: Placement) -> Option<PathBuf> {
        let chosen = match placement {
            Placement::Instance => self.preferences.default_folder.clone(),
            Placement::Tab | Placement::Split(_) => self
                .workspace
                .active_instance()
                .and_then(|instance| instance.folder.clone()),
        };
        let current = self.active_folder();
        chosen.filter(|folder| folder.is_dir()).or(current)
    }

    pub fn folder_of(&self, target: FolderTarget) -> Option<PathBuf> {
        match target {
            FolderTarget::Default => self.preferences.default_folder.clone(),
            FolderTarget::Instance(id) => {
                let instance = self.workspace.instance(id)?;
                let current = instance.active_tab().and_then(|tab| self.sessions.cwd(tab));
                instance.folder.clone().or(current)
            }
            FolderTarget::Tab(id) => self.sessions.cwd(id),
        }
    }

    pub fn active_folder(&self) -> Option<PathBuf> {
        let tab = self.workspace.active_tab()?;
        self.sessions.cwd(tab)
    }

    fn relaunch(&mut self, tabs: Vec<SessionId>, folder: PathBuf) -> Effects {
        let mut fresh = Vec::with_capacity(tabs.len());
        for old in tabs {
            let new = self.sessions.reserve();
            if self.workspace.replace_tab(old, new) {
                self.sessions.remove(old);
                fresh.push((new, Some(folder.clone())));
            }
        }
        self.queue_starts(fresh)
    }
}
