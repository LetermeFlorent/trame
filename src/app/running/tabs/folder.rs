use std::path::{Path, PathBuf};

use trame::files::{PickRequest, pick_folder};
use trame::session::SessionId;
use trame::ui::frame::EditTarget;
use trame::workspace::{Instance, InstanceId};

use super::Placement;
use crate::app::running::Running;
use crate::app::{UserEvent, window};

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

impl Running {
    pub(in crate::app::running) fn ask_folder(&self, target: FolderTarget, start: Option<PathBuf>) {
        let request = PickRequest {
            owner: window::owner(&self.window),
            title: self.strings.pick_folder.to_owned(),
            start,
        };
        let proxy = self.proxy.clone();
        pick_folder(request, move |folder| {
            let _ = proxy.send_event(UserEvent::Folder(target, folder));
        });
    }

    pub fn folder_picked(&mut self, target: FolderTarget, folder: PathBuf) {
        if let Some(chosen) = self.chosen_folder(target) {
            *chosen = Some(folder);
            return self.changed();
        }
        match target {
            FolderTarget::Default => {
                let next = self
                    .preferences
                    .with(|next| next.default_folder = Some(folder));
                self.set_preferences(next);
            }
            FolderTarget::Instance(id) => {
                self.workspace.set_instance_folder(id, Some(folder.clone()));
                let tabs = self.workspace.instance(id).map(Instance::tabs);
                self.relaunch(tabs.unwrap_or_default(), &folder);
            }
            FolderTarget::Tab(id) => self.relaunch(vec![id], &folder),
        }
    }

    pub(super) fn start_folder(&self, placement: Placement) -> Option<PathBuf> {
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

    pub(in crate::app::running) fn folder_of(&self, target: FolderTarget) -> Option<PathBuf> {
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

    fn relaunch(&mut self, tabs: Vec<SessionId>, folder: &Path) {
        let mut fresh = Vec::with_capacity(tabs.len());
        for old in tabs {
            let new = self.sessions.reserve();
            if self.workspace.replace_tab(old, new) {
                self.sessions.remove(old);
                fresh.push(new);
            }
        }
        self.relayout();
        for id in fresh {
            if let Err(error) = self.start_shell(id, Some(folder), None) {
                eprintln!("trame: the shell could not start in the new folder: {error}");
            }
        }
        self.changed();
    }
}
