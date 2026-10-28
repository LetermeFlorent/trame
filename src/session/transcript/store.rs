use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::MAX_TRANSCRIPT_BYTES;
use super::folder::{file_name, is_valid_key, report_unless_missing};
use super::round::{Job, Round};
use crate::session::SessionId;
use crate::term::Terminal;

pub const TRANSCRIPT_SAVE_INTERVAL: Duration = Duration::from_secs(60);

pub struct Transcripts {
    folder: Option<PathBuf>,
    stamp: u64,
    keys: HashMap<SessionId, String>,
    written: HashMap<SessionId, u64>,
    next_save: Instant,
    round: Option<Round>,
}

impl Transcripts {
    pub fn new(folder: Option<PathBuf>) -> Self {
        Self {
            folder,
            stamp: launch_stamp(),
            keys: HashMap::new(),
            written: HashMap::new(),
            next_save: Instant::now() + TRANSCRIPT_SAVE_INTERVAL,
            round: None,
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

    pub fn save(
        &mut self,
        tabs: &[SessionId],
        terminal_of: impl Fn(SessionId) -> Option<Arc<Mutex<Terminal>>>,
    ) {
        if self
            .round
            .as_ref()
            .is_some_and(|round| !round.is_finished())
        {
            return;
        }
        self.flush();
        self.keep_only(tabs);
        let Some(folder) = self.folder.clone() else {
            return;
        };
        let jobs = tabs
            .iter()
            .filter_map(|id| {
                Some(Job {
                    id: *id,
                    path: self.path_of(self.key(*id)?)?,
                    terminal: terminal_of(*id)?,
                    written: self.written.get(id).copied(),
                })
            })
            .collect();
        let kept = self.keys.values().map(|key| file_name(key)).collect();
        self.round = Round::start(jobs, folder, kept);
    }

    pub fn flush(&mut self) {
        let Some(round) = self.round.take() else {
            return;
        };
        for (id, generation) in round.finish() {
            if self.keys.contains_key(&id) {
                self.written.insert(id, generation);
            }
        }
    }

    pub fn clear(&mut self) {
        if let Some(round) = self.round.take() {
            round.cancel();
        }
        self.written.clear();
        let Some(folder) = &self.folder else {
            return;
        };
        if let Err(error) = fs::remove_dir_all(folder) {
            report_unless_missing("deleted", &error);
        }
    }

    fn keep_only(&mut self, tabs: &[SessionId]) {
        self.assign(tabs);
        let live: HashSet<SessionId> = tabs.iter().copied().collect();
        self.keys.retain(|id, _| live.contains(id));
        self.written.retain(|id, _| live.contains(id));
    }

    fn path_of(&self, key: &str) -> Option<PathBuf> {
        let folder = self.folder.as_ref().filter(|_| is_valid_key(key))?;
        Some(folder.join(file_name(key)))
    }
}

fn launch_stamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}
