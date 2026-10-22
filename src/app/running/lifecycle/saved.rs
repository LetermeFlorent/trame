use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use trame::session::SessionId;
use trame::workspace::{Snapshot, Workspace};

use crate::app::running::Running;
use crate::app::running::tabs::FALLBACK_GRID;
use crate::app::{persist, window};

type SavedCwds = HashMap<SessionId, Option<PathBuf>>;

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
        let sessions = &self.sessions;
        let mut snapshot = self
            .workspace
            .snapshot(self.font_size, &|id| sessions.cwd(id));
        snapshot.window = self.saver.remember_window(window::snapshot(&self.window));
        self.saver.save(&snapshot);
    }

    pub(super) fn restore_session(&mut self, saved: Option<&Snapshot>) -> bool {
        let Some(saved) = saved else {
            return false;
        };
        let mut cwds = SavedCwds::new();
        let sessions = &mut self.sessions;
        let restored = Workspace::restore(saved, &mut |tab| {
            let id = sessions.reserve();
            cwds.insert(id, tab.cwd.clone());
            id
        });
        match restored {
            Ok(workspace) => self.workspace = workspace,
            Err(error) => {
                eprintln!("trame: {error}, starting a fresh session");
                return false;
            }
        }
        self.set_font_size(saved.font_size);
        self.relayout();
        self.spawn_restored(cwds);
        if self.workspace.is_empty() {
            return false;
        }
        self.saver.baseline(&self.workspace);
        self.changed();
        true
    }

    fn spawn_restored(&mut self, mut cwds: SavedCwds) {
        for id in self.workspace.tabs() {
            let cwd = existing_or_home(cwds.remove(&id).flatten());
            let size = self.grid_size_of(id).unwrap_or(FALLBACK_GRID);
            let colors = self.theme.reported_colors();
            if let Err(error) = self.sessions.spawn(id, cwd.as_deref(), size, colors) {
                eprintln!("trame: a saved shell could not start: {error}");
                self.workspace.close_tab(id);
            }
        }
    }
}

fn existing_or_home(saved: Option<PathBuf>) -> Option<PathBuf> {
    match saved {
        Some(path) if path.is_dir() => Some(path),
        Some(_) => persist::home_dir(),
        None => None,
    }
}
