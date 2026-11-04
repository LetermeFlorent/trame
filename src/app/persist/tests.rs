use std::fs;
use std::io;

use trame::preferences::Preferences;
use trame::session::Transcripts;
use trame::workspace::{Snapshot, TabSnapshot, Workspace};

use super::{Role, Saver, load_from};
use crate::app::scratch::Scratch;

fn claim(scratch: &Scratch) -> Role {
    Role::claim(Some(scratch.path().to_owned()))
}

fn snapshot() -> Snapshot {
    Workspace::new().snapshot(Preferences::default(), &|_| TabSnapshot::default())
}

#[test]
fn the_first_process_owns_the_session_files() {
    let scratch = Scratch::new("persist-first");
    let first = claim(&scratch);
    let second = claim(&scratch);
    assert!(first.owns());
    assert!(!second.owns());
    assert_eq!(second.session_path(), None);
}

#[test]
fn a_guest_process_never_prunes() {
    let scratch = Scratch::new("persist-guest");
    let _owner = claim(&scratch);
    let guest = claim(&scratch);
    let folder = scratch.path().join("text");
    fs::create_dir_all(&folder).expect("text folder");
    let kept = folder.join("owner.vt");
    fs::write(&kept, b"owner text").expect("owner text");
    let mut transcripts = Transcripts::new(guest.transcript_dir());
    transcripts.save(&[], |_| None);
    transcripts.flush();
    assert!(kept.exists());
}

#[test]
fn a_restart_hands_the_session_files_to_the_next_process() {
    let scratch = Scratch::new("persist-handover");
    let mut first = claim(&scratch);
    let mut next = None;
    first
        .hand_over(|| {
            next = Some(claim(&scratch));
            Ok(())
        })
        .expect("the next process starts");
    assert!(!first.owns());
    assert!(next.is_some_and(|role| role.owns()));
}

#[test]
fn a_restart_that_does_not_start_keeps_the_session_files() {
    let scratch = Scratch::new("persist-failed");
    let mut role = claim(&scratch);
    let started = role.hand_over(|| Err(io::Error::other("no next process")));
    assert!(started.is_err());
    assert!(role.owns());
    assert!(!claim(&scratch).owns());
}

#[test]
fn a_guest_stays_a_guest_when_a_restart_fails() {
    let scratch = Scratch::new("persist-guest-restart");
    let owner = claim(&scratch);
    let mut guest = claim(&scratch);
    drop(owner);
    let started = guest.hand_over(|| Err(io::Error::other("no next process")));
    assert!(started.is_err());
    assert!(!guest.owns());
}

#[test]
fn an_unreadable_session_is_set_aside() {
    let scratch = Scratch::new("persist-unreadable");
    let mut role = claim(&scratch);
    let path = scratch.path().join("session.json");
    fs::write(&path, b"\xff").expect("a damaged file");
    assert!(load_from(&path, &mut role).is_none());
    assert!(path.with_extension("json.bad").exists());
    assert!(role.owns());
}

#[test]
fn a_session_that_cannot_be_set_aside_is_never_replaced() {
    let scratch = Scratch::new("persist-stuck");
    let mut role = claim(&scratch);
    let path = scratch.path().join("session.json");
    let taken = path.with_extension("json.bad").join("taken");
    fs::create_dir_all(taken).expect("a folder in the way");
    fs::write(&path, b"\xff").expect("a damaged file");
    assert!(load_from(&path, &mut role).is_none());
    assert!(!role.owns());
    assert_eq!(fs::read(&path).expect("the damaged file"), b"\xff");
}

#[test]
fn a_failed_save_stays_dirty() {
    let scratch = Scratch::new("persist-save");
    let mut saver = Saver::new(claim(&scratch));
    let blocker = scratch.path().join("session.json");
    fs::create_dir(&blocker).expect("a folder in the way");
    saver.touch();
    saver.save(&snapshot());
    assert!(saver.due().is_some());
    fs::remove_dir(&blocker).expect("the folder goes");
    saver.save(&snapshot());
    assert!(saver.due().is_none());
}

#[test]
fn a_failed_save_leaves_the_session_unsaved_until_a_save_works() {
    let scratch = Scratch::new("persist-unsaved");
    let mut saver = Saver::new(claim(&scratch));
    let blocker = scratch.path().join("session.json");
    fs::create_dir(&blocker).expect("a folder in the way");
    saver.touch();
    assert!(saver.keeps_session());
    saver.save(&snapshot());
    assert!(!saver.keeps_session());
    fs::remove_dir(&blocker).expect("the folder goes");
    saver.save(&snapshot());
    assert!(saver.keeps_session());
}

#[cfg(unix)]
#[test]
fn the_state_folder_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("persist-private");
    let fresh = scratch.path().join("trame");
    let _new = Role::claim(Some(fresh.clone()));
    let _old = claim(&scratch);
    for folder in [fresh.as_path(), scratch.path()] {
        let found = fs::metadata(folder).expect("the folder").permissions();
        assert_eq!(found.mode() & 0o777, 0o700);
    }
}
