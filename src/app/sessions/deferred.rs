use std::path::PathBuf;

use trame::session::SessionId;

use super::Sessions;

pub struct Waiting {
    pub transcript: Option<String>,
}

impl Sessions {
    pub fn defer(&mut self, id: SessionId, cwd: Option<PathBuf>, waiting: Waiting) {
        if let Some(path) = cwd {
            self.origins.insert(id, path);
        }
        self.waiting.insert(id, waiting);
    }

    pub fn is_waiting(&self, id: SessionId) -> bool {
        self.waiting.contains_key(&id)
    }

    pub fn take_waiting(&mut self, id: SessionId) -> Option<Waiting> {
        self.waiting.remove(&id)
    }
}
