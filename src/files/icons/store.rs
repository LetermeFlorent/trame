use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::load_icon;
use crate::log_error;

#[derive(Debug, Default)]
pub struct IconStore {
    dir: Option<PathBuf>,
    loaded: HashMap<u64, Option<Vec<u8>>>,
}

impl IconStore {
    pub fn new(dir: Option<PathBuf>) -> Self {
        Self {
            dir,
            loaded: HashMap::new(),
        }
    }

    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    pub fn load(&mut self, id: u64) {
        if self.loaded.contains_key(&id) {
            return;
        }
        let pixels = self.dir.as_deref().and_then(|dir| {
            load_icon(dir, id)
                .inspect_err(|error| log_error!("the icon {id:016x} could not be read: {error}"))
                .ok()
        });
        self.loaded.insert(id, pixels);
    }

    pub fn reload(&mut self, id: u64) {
        self.loaded.remove(&id);
        self.load(id);
    }

    pub fn pixels(&self, id: u64) -> Option<&[u8]> {
        self.loaded.get(&id)?.as_deref()
    }
}
