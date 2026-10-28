use std::fs;
use std::io;

use trame::session::Transcripts;

use super::Role;
use crate::app::scratch::Scratch;

fn claim(scratch: &Scratch) -> Role {
    Role::claim(Some(scratch.path().to_owned()))
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
    Transcripts::new(guest.transcript_dir()).prune(&[]);
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
