use std::collections::HashMap;
use std::path::PathBuf;

use crate::desk::{Desk, Waiting};
use crate::files::home_dir;
use crate::log_error;
use crate::preferences::Recall;
use crate::session::SessionId;
use crate::workspace::{Snapshot, TabSnapshot, Workspace};

type SavedTabs = HashMap<SessionId, TabSnapshot>;

impl Desk {
    pub fn restore(&mut self, saved: Option<&Snapshot>) -> bool {
        let Some(saved) = saved.and_then(Snapshot::recalled) else {
            return false;
        };
        let mut tabs = SavedTabs::new();
        let sessions = &mut self.sessions;
        let restored = Workspace::restore(&saved, &mut |tab| {
            let id = sessions.reserve();
            tabs.insert(id, tab.clone());
            id
        });
        match restored {
            Ok(workspace) => {
                self.workspace = workspace;
                self.workspace.set_numbering(self.preferences.numbering);
            }
            Err(error) => {
                log_error!("{error}, starting a fresh session");
                return false;
            }
        }
        self.defer_restored(tabs, saved.resume);
        true
    }

    pub fn settle_restore(&mut self) -> bool {
        if self.workspace.is_empty() {
            return false;
        }
        self.saver.baseline(&self.workspace);
        true
    }

    fn defer_restored(&mut self, mut tabs: SavedTabs, resume: bool) {
        let keeps_text = resume || self.preferences.recalls(Recall::Text);
        for id in self.workspace.tabs() {
            let saved = tabs.remove(&id).unwrap_or_default();
            let waiting = Waiting {
                transcript: saved.transcript.clone().filter(|_| keeps_text),
            };
            self.sessions
                .defer(id, existing_or_home(saved.cwd), waiting);
            self.transcripts.adopt(id, saved.transcript.as_deref());
        }
    }
}

fn existing_or_home(saved: Option<PathBuf>) -> Option<PathBuf> {
    match saved {
        Some(path) if path.is_dir() => Some(path),
        Some(_) => home_dir(),
        None => None,
    }
}
