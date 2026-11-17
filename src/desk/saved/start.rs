use std::path::PathBuf;

use crate::desk::{Desk, Effects, Placement};
use crate::preferences::Recall;

impl Desk {
    pub fn open_first(&mut self, restored: bool, folder: Option<PathBuf>) -> Effects {
        if restored && folder.is_none() {
            return Effects::default();
        }
        self.open_at(Placement::Instance, folder)
    }

    pub fn settle_start(&mut self) -> bool {
        if self.workspace.is_empty() {
            return false;
        }
        if !self.preferences.recalls(Recall::Text) {
            self.transcripts.clear();
        }
        true
    }
}
