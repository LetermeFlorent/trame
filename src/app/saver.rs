use std::path::PathBuf;
use std::time::{Duration, Instant};

use trame::workspace::{Snapshot, WindowSnapshot, Workspace};

use super::persist::{session_path, write};

const SAVE_INTERVAL: Duration = Duration::from_secs(1);

pub struct Saver {
    path: Option<PathBuf>,
    last_layout: Option<Workspace>,
    last_write: Option<Instant>,
    last_window: Option<WindowSnapshot>,
    dirty: bool,
}

impl Saver {
    pub fn new() -> Self {
        Self {
            path: session_path(),
            last_layout: None,
            last_write: None,
            last_window: None,
            dirty: false,
        }
    }

    pub fn baseline(&mut self, workspace: &Workspace) {
        self.last_layout = Some(workspace.clone());
    }

    pub fn touch(&mut self) {
        self.dirty = true;
    }

    pub fn watch(&mut self, workspace: &Workspace) {
        if self.last_layout.as_ref() != Some(workspace) {
            self.last_layout = Some(workspace.clone());
            self.dirty = true;
        }
    }

    pub fn remember_window(&mut self, window: Option<WindowSnapshot>) -> Option<WindowSnapshot> {
        if window.is_some() {
            self.last_window = window;
        }
        self.last_window
    }

    pub fn due(&self) -> Option<Instant> {
        if !self.dirty {
            return None;
        }
        Some(
            self.last_write
                .map_or_else(Instant::now, |last| last + SAVE_INTERVAL),
        )
    }

    pub fn save(&mut self, snapshot: &Snapshot) {
        self.dirty = false;
        self.last_write = Some(Instant::now());
        let Some(path) = &self.path else {
            return;
        };
        if let Err(error) = write(path, snapshot) {
            eprintln!("trame: the session could not be saved: {error}");
        }
    }
}
