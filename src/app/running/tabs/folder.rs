use std::path::PathBuf;

use trame::desk::FolderTarget;
use trame::files::{PickRequest, pick_folder};

use crate::app::running::Running;
use crate::app::{UserEvent, window};

impl Running {
    pub(in crate::app::running) fn ask_folder(&self, target: FolderTarget, start: Option<PathBuf>) {
        let request = PickRequest {
            owner: window::owner(&self.window),
            title: self.desk.strings.pick_folder.to_owned(),
            start,
        };
        let proxy = self.proxy.clone();
        let origin = self.window.id();
        pick_folder(request, move |folder| {
            let _ = proxy.send_event(UserEvent::Folder(origin, target, folder));
        });
    }

    pub fn folder_picked(&mut self, target: FolderTarget, folder: PathBuf) {
        if let Some(chosen) = self.chosen_folder(target) {
            *chosen = Some(folder);
            return self.changed();
        }
        let effects = self.desk.folder_picked(target, folder);
        self.apply(effects);
    }
}
