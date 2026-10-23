use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use trame::files::home_dir;
use trame::preferences::Recall;
use trame::session::SessionId;
use trame::workspace::{Snapshot, TabSnapshot, Workspace};

use crate::app::running::Running;
use crate::app::sessions::Waiting;
use crate::app::window;

type SavedTabs = HashMap<SessionId, TabSnapshot>;

impl Running {
    pub(super) fn save_deadline(&mut self) -> Option<Instant> {
        self.saver.watch(&self.workspace);
        let due = self.saver.due()?;
        if due > Instant::now() {
            return Some(due);
        }
        self.save_session(false);
        None
    }

    pub(super) fn save_session(&mut self, resume: bool) {
        self.transcripts.assign(&self.workspace.tabs());
        let sessions = &self.sessions;
        let transcripts = &self.transcripts;
        let keeps_text = resume || self.preferences.recalls(Recall::Text);
        let describe = |id: SessionId| TabSnapshot {
            cwd: sessions.cwd(id),
            transcript: transcripts
                .key(id)
                .filter(|_| keeps_text)
                .map(str::to_owned),
            ..TabSnapshot::default()
        };
        let mut snapshot = self.workspace.snapshot(self.preferences.clone(), &describe);
        snapshot.resume = resume;
        snapshot.window = self.saver.remember_window(window::snapshot(&self.window));
        self.saver.save(&snapshot);
    }

    pub(super) fn restore_session(&mut self, saved: Option<&Snapshot>) -> bool {
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
            Ok(workspace) => self.workspace = workspace,
            Err(error) => {
                eprintln!("trame: {error}, starting a fresh session");
                return false;
            }
        }
        self.relayout();
        self.defer_restored(tabs, saved.resume);
        self.start_shown();
        if self.workspace.is_empty() {
            return false;
        }
        self.saver.baseline(&self.workspace);
        self.changed();
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
