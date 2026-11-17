use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use trame::desk::{Desk, Grids, Placement};
use trame::host::persist::{Role, load_from};
use trame::preferences::Recall;
use trame::workspace::WindowSnapshot;

use crate::sandbox::Sandbox;
use crate::support::{desk, desk_in};

const LATER: Duration = Duration::from_secs(3600);

fn no_window() -> Option<WindowSnapshot> {
    None
}

fn started(sandbox: &Sandbox) -> (Desk, PathBuf) {
    let mut desk = desk_in(sandbox);
    let _ = desk.open(Placement::Tab);
    desk.start_queued(&Grids::default());
    let path = desk.saver.slot().session_path().expect("a session file");
    (desk, path)
}

#[test]
fn a_restarting_desk_saves_nothing_on_exit() {
    let sandbox = Sandbox::new("desk-restarting");
    let (mut desk, path) = started(&sandbox);
    desk.update.restarting = true;
    assert!(desk.save_on_exit(None).is_empty());
    assert!(!path.exists());
    desk.update.restarting = false;
    let _ = desk.save_on_exit(None);
    assert!(path.exists());
}

#[test]
fn a_restart_saves_every_transcript_and_resumes() {
    let sandbox = Sandbox::new("desk-restart");
    let (mut desk, path) = started(&sandbox);
    desk.preferences.restore_text = false;
    assert!(!desk.preferences.recalls(Recall::Text));
    let _ = desk.save_for_restart(None);
    let snapshot = load_from(&path, &mut Role::Guest).expect("a saved session");
    assert!(snapshot.resume);
    let text = fs::read_to_string(&path).expect("a readable session");
    assert!(text.contains("\"transcript\""));
}

#[test]
fn a_quit_without_text_recall_keeps_no_transcript() {
    let sandbox = Sandbox::new("desk-quit-text");
    let (mut desk, path) = started(&sandbox);
    desk.preferences.restore_text = false;
    let _ = desk.save_on_exit(None);
    let snapshot = load_from(&path, &mut Role::Guest).expect("a saved session");
    assert!(!snapshot.resume);
    let text = fs::read_to_string(&path).expect("a readable session");
    assert!(!text.contains("\"transcript\""));
}

#[test]
fn the_save_waits_for_its_due_time() {
    let sandbox = Sandbox::new("desk-save-due");
    let (mut desk, path) = started(&sandbox);
    let first = desk.save_deadline(Instant::now() + LATER, no_window);
    assert_eq!(first.value, None);
    assert!(path.exists());
    fs::remove_file(&path).expect("the session file is removed");
    let _ = desk.open(Placement::Tab);
    let now = Instant::now();
    let waiting = desk.save_deadline(now, no_window);
    let due = waiting.value.expect("a save is due later");
    assert!(due > now);
    assert!(!path.exists());
    let saved = desk.save_deadline(due, no_window);
    assert_eq!(saved.value, None);
    assert!(path.exists());
}

#[test]
fn a_guest_desk_keeps_no_transcript_deadline() {
    let mut desk = desk();
    assert!(desk.transcript_deadline(Instant::now()).is_none());
}

#[test]
fn an_owner_desk_schedules_its_transcripts() {
    let sandbox = Sandbox::new("desk-text-due");
    let (mut desk, _) = started(&sandbox);
    assert!(desk.preferences.recalls(Recall::Text));
    assert!(desk.transcript_deadline(Instant::now()).is_some());
}
