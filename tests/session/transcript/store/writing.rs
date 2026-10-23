use std::fs;
use std::time::Instant;

use trame::session::{TRANSCRIPT_SAVE_INTERVAL, Transcripts};

use super::{FIRST, SECOND, Sandbox, key_of};

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
