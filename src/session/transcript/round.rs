use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use super::folder::remove_stale;
use super::{MAX_TRANSCRIPT_BYTES, TranscriptCopy};
use crate::files::write_atomic;
use crate::log_error;
use crate::session::{SessionId, lock};
use crate::term::Terminal;

pub(super) struct Job {
    pub(super) id: SessionId,
    pub(super) path: PathBuf,
    pub(super) terminal: Arc<Mutex<Terminal>>,
    pub(super) written: Option<u64>,
}

pub(super) struct Round {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<Vec<(SessionId, u64)>>,
}

impl Round {
    pub(super) fn start(jobs: Vec<Job>, folder: PathBuf, kept: HashSet<String>) -> Option<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let spawned = thread::Builder::new()
            .name("trame-text".into())
            .spawn(move || run(&jobs, &folder, &kept, &stopped));
        match spawned {
            Ok(handle) => Some(Self { stop, handle }),
            Err(error) => {
                log_error!("the tab texts could not be saved: {error}");
                None
            }
        }
    }

    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) fn finish(self) -> Vec<(SessionId, u64)> {
        self.handle.join().unwrap_or_else(|_| {
            log_error!("saving the tab texts stopped halfway");
            Vec::new()
        })
    }

    pub(super) fn cancel(self) {
        self.stop.store(true, Ordering::Relaxed);
        self.finish();
    }
}

fn run(
    jobs: &[Job],
    folder: &Path,
    kept: &HashSet<String>,
    stop: &AtomicBool,
) -> Vec<(SessionId, u64)> {
    let mut written = Vec::new();
    for job in jobs {
        if stop.load(Ordering::Relaxed) {
            return written;
        }
        if job.written == Some(lock(&job.terminal).generation()) {
            continue;
        }
        let copy = TranscriptCopy::take(&job.terminal);
        match write_atomic(&job.path, &copy.encode(MAX_TRANSCRIPT_BYTES)) {
            Ok(()) => written.push((job.id, copy.generation())),
            Err(error) => log_error!("a tab text could not be saved: {error}"),
        }
    }
    remove_stale(folder, kept);
    written
}
