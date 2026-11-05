use std::path::PathBuf;
use std::time::{Duration, Instant};

use trame::log_error;
use trame::workspace::{Snapshot, WindowSnapshot, Workspace};

use super::{Slot, write};

const SAVE_INTERVAL: Duration = Duration::from_secs(1);

pub struct Saver {
    slot: Slot,
    last_layout: Option<Workspace>,
    last_write: Option<Instant>,
    last_window: Option<WindowSnapshot>,
    dirty: bool,
    failing: bool,
}

impl Saver {
    pub fn new(slot: Slot) -> Self {
        Self {
            slot,
            last_layout: None,
            last_write: None,
            last_window: None,
            dirty: false,
            failing: false,
        }
    }

    pub fn owns(&self) -> bool {
        self.slot.owns()
    }

    pub fn keeps_session(&self) -> bool {
        self.slot.owns() && !self.failing
    }

    pub fn transcript_dir(&self) -> Option<PathBuf> {
        self.slot.transcript_dir()
    }

    pub fn slot(&self) -> &Slot {
        &self.slot
    }

    pub fn stop(&mut self) {
        self.slot = Slot::new(None, self.slot.index());
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
        if !self.dirty || !self.slot.owns() {
            return None;
        }
        Some(
            self.last_write
                .map_or_else(Instant::now, |last| last + SAVE_INTERVAL),
        )
    }

    pub fn save(&mut self, snapshot: &Snapshot) {
        self.last_write = Some(Instant::now());
        let Some(path) = self.slot.session_path() else {
            return;
        };
        match write(&path, snapshot) {
            Ok(()) => {
                self.dirty = false;
                self.failing = false;
            }
            Err(error) if !self.failing => {
                log_error!("the session could not be saved: {error}");
                self.failing = true;
            }
            Err(_) => {}
        }
    }
}
