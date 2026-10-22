use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

use trame::session::{MAX_TRANSCRIPT_BYTES, SessionId, TRANSCRIPT_SAVE_INTERVAL, Transcripts};

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

#[test]
fn written_text_reads_back_and_marks_the_generation() {
    let sandbox = Sandbox::new("round-trip");
    let mut transcripts = sandbox.transcripts();
    transcripts.assign(&[FIRST]);
    transcripts.write(FIRST, 7, b"hello").unwrap();
    let key = key_of(&transcripts, FIRST);
    assert_eq!(transcripts.read(&key), Some(b"hello".to_vec()));
    assert!(transcripts.is_current(FIRST, 7));
    assert!(!transcripts.is_current(FIRST, 8));
}

#[test]
fn unsafe_keys_are_refused() {
    let sandbox = Sandbox::new("keys");
    fs::create_dir_all(sandbox.text()).unwrap();
    fs::write(sandbox.root.join("escape.vt"), b"outside").unwrap();
    let mut transcripts = sandbox.transcripts();
    let too_long = "a".repeat(41);
    for key in ["", "../escape", "a/b", "a.b", too_long.as_str()] {
        assert_eq!(transcripts.read(key), None, "{key:?}");
        transcripts.adopt(FIRST, Some(key));
        assert_eq!(transcripts.key(FIRST), None, "{key:?}");
    }
    let longest = "a".repeat(40);
    transcripts.adopt(FIRST, Some(&longest));
    assert_eq!(transcripts.key(FIRST), Some(longest.as_str()));
}

#[test]
fn adopted_keys_survive_assign() {
    let mut transcripts = Transcripts::new(None);
    transcripts.adopt(FIRST, Some("saved-1"));
    transcripts.assign(&[FIRST, SECOND]);
    assert_eq!(transcripts.key(FIRST), Some("saved-1"));
    let fresh = key_of(&transcripts, SECOND);
    assert!(fresh.ends_with("-2"), "{fresh}");
}

#[test]
fn prune_removes_closed_tabs_and_stray_files() {
    let sandbox = Sandbox::new("prune");
    let mut transcripts = sandbox.transcripts();
    transcripts.assign(&[FIRST, SECOND]);
    transcripts.write(FIRST, 1, b"first").unwrap();
    transcripts.write(SECOND, 1, b"second").unwrap();
    fs::write(sandbox.text().join("stray.vt"), b"old").unwrap();
    transcripts.prune(&[FIRST]);
    let kept = format!("{}.vt", key_of(&transcripts, FIRST));
    assert_eq!(sandbox.files(), vec![kept]);
    assert_eq!(transcripts.key(SECOND), None);
    assert!(!transcripts.is_current(SECOND, 1));
}

#[test]
fn clear_removes_every_saved_text() {
    let sandbox = Sandbox::new("clear");
    let mut transcripts = sandbox.transcripts();
    transcripts.assign(&[FIRST]);
    transcripts.write(FIRST, 3, b"gone").unwrap();
    transcripts.clear();
    assert!(!sandbox.text().exists());
    assert!(!transcripts.is_current(FIRST, 3));
    assert_eq!(transcripts.read(&key_of(&transcripts, FIRST)), None);
}

#[test]
fn oversized_files_are_ignored() {
    let sandbox = Sandbox::new("oversized");
    fs::create_dir_all(sandbox.text()).unwrap();
    fs::write(
        sandbox.text().join("big.vt"),
        vec![b'x'; MAX_TRANSCRIPT_BYTES + 1],
    )
    .unwrap();
    assert_eq!(sandbox.transcripts().read("big"), None);
}

#[test]
fn without_a_folder_nothing_is_written() {
    let mut transcripts = Transcripts::new(None);
    transcripts.assign(&[FIRST]);
    transcripts.write(FIRST, 1, b"lost").unwrap();
    assert!(!transcripts.is_current(FIRST, 1));
    assert_eq!(transcripts.read(&key_of(&transcripts, FIRST)), None);
}

#[test]
fn first_save_waits_a_full_interval() {
    let before = Instant::now();
    let mut transcripts = Transcripts::new(None);
    assert!(transcripts.due() >= before + TRANSCRIPT_SAVE_INTERVAL);
    let rescheduled = Instant::now();
    transcripts.schedule();
    assert!(transcripts.due() >= rescheduled + TRANSCRIPT_SAVE_INTERVAL);
}
