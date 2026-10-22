use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::MAX_TRANSCRIPT_BYTES;
use crate::files::write_atomic;
use crate::session::SessionId;

pub const TRANSCRIPT_SAVE_INTERVAL: Duration = Duration::from_secs(60);

const EXTENSION: &str = "vt";
const MAX_KEY_LENGTH: usize = 40;

pub struct Transcripts {
    folder: Option<PathBuf>,
    stamp: u64,
    keys: HashMap<SessionId, String>,
    written: HashMap<SessionId, u64>,
    next_save: Instant,
}

impl Transcripts {
    pub fn new(folder: Option<PathBuf>) -> Self {
        Self {
            folder,
            stamp: launch_stamp(),
            keys: HashMap::new(),
            written: HashMap::new(),
            next_save: Instant::now() + TRANSCRIPT_SAVE_INTERVAL,
        }
    }

    pub fn read(&self, key: &str) -> Option<Vec<u8>> {
        let path = self.path_of(key)?;
        let limit = MAX_TRANSCRIPT_BYTES as u64 + 1;
        let mut bytes = Vec::new();
        let read = File::open(path).and_then(|file| file.take(limit).read_to_end(&mut bytes));
        if let Err(error) = read {
            report_unless_missing("read", &error);
            return None;
        }
        (bytes.len() <= MAX_TRANSCRIPT_BYTES).then_some(bytes)
    }

    pub fn adopt(&mut self, id: SessionId, key: Option<&str>) {
        if let Some(key) = key.filter(|key| is_valid_key(key)) {
            self.keys.insert(id, key.to_owned());
        }
    }

    pub fn assign(&mut self, tabs: &[SessionId]) {
        let stamp = self.stamp;
        for id in tabs {
            self.keys
                .entry(*id)
                .or_insert_with(|| format!("{stamp:x}-{}", id.0));
        }
    }

    pub fn key(&self, id: SessionId) -> Option<&str> {
        self.keys.get(&id).map(String::as_str)
    }

    pub fn due(&self) -> Instant {
        self.next_save
    }

    pub fn schedule(&mut self) {
        self.next_save = Instant::now() + TRANSCRIPT_SAVE_INTERVAL;
    }

    pub fn is_current(&self, id: SessionId, generation: u64) -> bool {
        self.written.get(&id) == Some(&generation)
    }

    pub fn write(&mut self, id: SessionId, generation: u64, bytes: &[u8]) -> io::Result<()> {
        let Some(path) = self.key(id).and_then(|key| self.path_of(key)) else {
            return Ok(());
        };
        write_atomic(&path, bytes)?;
        self.written.insert(id, generation);
        Ok(())
    }

    pub fn prune(&mut self, live: &[SessionId]) {
        let live: HashSet<SessionId> = live.iter().copied().collect();
        self.keys.retain(|id, _| live.contains(id));
        self.written.retain(|id, _| live.contains(id));
        let Some(entries) = self.folder.as_ref().and_then(|dir| fs::read_dir(dir).ok()) else {
            return;
        };
        let kept: HashSet<String> = self.keys.values().map(|key| file_name(key)).collect();
        for entry in entries.flatten() {
            let stale = entry.file_type().is_ok_and(|kind| kind.is_file())
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| !kept.contains(name));
            if stale && let Err(error) = fs::remove_file(entry.path()) {
                report_unless_missing("deleted", &error);
            }
        }
    }

    pub fn clear(&mut self) {
        self.written.clear();
        let Some(folder) = &self.folder else {
            return;
        };
        if let Err(error) = fs::remove_dir_all(folder) {
            report_unless_missing("deleted", &error);
        }
    }

    fn path_of(&self, key: &str) -> Option<PathBuf> {
        let folder = self.folder.as_ref().filter(|_| is_valid_key(key))?;
        Some(folder.join(file_name(key)))
    }
}

fn file_name(key: &str) -> String {
    format!("{key}.{EXTENSION}")
}

fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= MAX_KEY_LENGTH
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn launch_stamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

fn report_unless_missing(action: &str, error: &io::Error) {
    if error.kind() != ErrorKind::NotFound {
        eprintln!("trame: a saved tab text could not be {action}: {error}");
    }
}
