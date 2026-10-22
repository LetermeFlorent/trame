use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use trame::session::SessionId;
use trame::workspace::{Snapshot, TabSnapshot, Workspace};

use crate::app::running::Running;
use crate::app::running::tabs::FALLBACK_GRID;
use crate::app::sessions::ShellStart;
use crate::app::{persist, window};

type SavedTabs = HashMap<SessionId, TabSnapshot>;

impl Running {
    pub(super) fn save_deadline(&mut self) -> Option<Instant> {
        self.saver.watch(&self.workspace);
        let due = self.saver.due()?;
        if due > Instant::now() {
            return Some(due);
        }
        self.save_session();
        None
    }

    pub fn save_session(&mut self) {
        self.transcripts.assign(&self.workspace.tabs());
        let sessions = &self.sessions;
        let transcripts = &self.transcripts;
        let keeps_text = self.preferences.restore_text;
        let describe = |id: SessionId| TabSnapshot {
            cwd: sessions.cwd(id),
            transcript: transcripts
                .key(id)
                .filter(|_| keeps_text)
                .map(str::to_owned),
            ..TabSnapshot::default()
        };
        let mut snapshot = self.workspace.snapshot(self.preferences, &describe);
        snapshot.window = self.saver.remember_window(window::snapshot(&self.window));
        self.saver.save(&snapshot);
    }

    pub(super) fn restore_session(&mut self, saved: Option<&Snapshot>) -> bool {
        let Some(saved) = saved else {
            return false;
        };
        let mut tabs = SavedTabs::new();
        let sessions = &mut self.sessions;
        let restored = Workspace::restore(saved, &mut |tab| {
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
        self.spawn_restored(tabs);
        if self.workspace.is_empty() {
            return false;
        }
        self.saver.baseline(&self.workspace);
        self.changed();
        true
    }

    fn spawn_restored(&mut self, mut tabs: SavedTabs) {
        for id in self.workspace.tabs() {
            let saved = tabs.remove(&id).unwrap_or_default();
            let cwd = existing_or_home(saved.cwd);
            let text = self.saved_text(saved.transcript.as_deref());
            let start = ShellStart {
                cwd: cwd.as_deref(),
                size: self.grid_size_of(id).unwrap_or(FALLBACK_GRID),
                colors: self.theme.reported_colors(),
                transcript: &text,
            };
            match self.sessions.spawn(id, &start) {
                Ok(()) => self.transcripts.adopt(id, saved.transcript.as_deref()),
                Err(error) => {
                    eprintln!("trame: a saved shell could not start: {error}");
                    self.workspace.close_tab(id);
                }
            }
        }
    }

    fn saved_text(&self, key: Option<&str>) -> Vec<u8> {
        key.filter(|_| self.preferences.restore_text)
            .and_then(|key| self.transcripts.read(key))
            .unwrap_or_default()
    }
}

fn existing_or_home(saved: Option<PathBuf>) -> Option<PathBuf> {
    match saved {
        Some(path) if path.is_dir() => Some(path),
        Some(_) => persist::home_dir(),
        None => None,
    }
}
