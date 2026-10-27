use std::path::PathBuf;

use trame::files::{home_dir, short_path};
use trame::ui::frame::{EditTarget, FormAction, MessageRequest};

use super::{EditMode, Editing, Running};
use crate::app::running::FolderTarget;

const FOLDER_ACTIONS: [FormAction; 2] = [FormAction::Change, FormAction::Apply];

impl Running {
    pub(super) fn folder_mode(&self, target: EditTarget) -> EditMode {
        let original = self.folder_of(target.into());
        EditMode::Folder {
            chosen: original.clone(),
            original,
        }
    }

    pub(super) fn place_folder(&mut self, shown: Option<String>) {
        let strings = self.strings;
        let request = MessageRequest {
            title: strings.change_folder,
            lines: &[shown.as_deref().unwrap_or(strings.pick_folder)],
            actions: &FOLDER_ACTIONS,
            strings,
        };
        self.frame.place_message(&mut self.text, &request);
    }

    pub(in crate::app::running) fn chosen_folder(
        &mut self,
        target: FolderTarget,
    ) -> Option<&mut Option<PathBuf>> {
        let editing = self.editing.as_mut()?;
        match &mut editing.mode {
            EditMode::Folder { chosen, .. } if FolderTarget::from(editing.target) == target => {
                Some(chosen)
            }
            _ => None,
        }
    }

    pub(super) fn change_chosen_folder(&self) {
        if let Some(Editing {
            target,
            mode: EditMode::Folder { chosen, .. },
            ..
        }) = &self.editing
        {
            self.ask_folder((*target).into(), chosen.clone());
        }
    }
}

pub(super) fn shown_folder(folder: Option<&PathBuf>) -> Option<String> {
    folder.map(|folder| short_path(folder, home_dir().as_deref()))
}
