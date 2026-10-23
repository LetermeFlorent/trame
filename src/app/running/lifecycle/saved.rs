use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use trame::files::home_dir;
use trame::preferences::Recall;
use trame::session::SessionId;
use trame::workspace::{Snapshot, TabSnapshot, Workspace};

use crate::app::running::Running;
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
        self.spawn_restored(tabs, saved.resume);
        if self.workspace.is_empty() {
            return false;
        }
        self.saver.baseline(&self.workspace);
        self.changed();
        true
    }

    fn spawn_restored(&mut self, mut tabs: SavedTabs, resume: bool) {
        let keeps_text = resume || self.preferences.recalls(Recall::Text);
        for id in self.workspace.tabs() {
            let saved = tabs.remove(&id).unwrap_or_default();
            let cwd = existing_or_home(saved.cwd);
            let key = saved.transcript.as_deref().filter(|_| keeps_text);
            let text = key.map(|key| self.saved_text(key));
            match self.start_shell(id, cwd.as_deref(), text.as_deref()) {
                Ok(()) => self.transcripts.adopt(id, saved.transcript.as_deref()),
                Err(error) => eprintln!("trame: a saved shell could not start: {error}"),
            }
        }
    }

    fn saved_text(&self, key: &str) -> Vec<u8> {
        self.transcripts.read(key).unwrap_or_default()
    }
}

fn existing_or_home(saved: Option<PathBuf>) -> Option<PathBuf> {
    match saved {
        Some(path) if path.is_dir() => Some(path),
        Some(_) => home_dir(),
        None => None,
    }
}
