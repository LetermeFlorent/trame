mod reading;
mod writing;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use trame::session::{SessionId, Transcripts};

const FIRST: SessionId = SessionId(1);
const SECOND: SessionId = SessionId(2);

struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let root = env::temp_dir().join(format!("trame-test-{}-{name}", process::id()));
        let _ = fs::remove_dir_all(&root);
        Self { root }
    }

    fn text(&self) -> PathBuf {
        self.root.join("text")
    }

    fn transcripts(&self) -> Transcripts {
        Transcripts::new(Some(self.text()))
    }

    fn files(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.text())
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn key_of(transcripts: &Transcripts, id: SessionId) -> String {
    transcripts
        .key(id)
        .expect("tab should have a key")
        .to_owned()
}
