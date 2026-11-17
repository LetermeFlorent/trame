use std::cell::RefCell;

use trame::host::update::UpdateStatus;
use trame::host::update::watch::{offer, poll_releases};

use crate::update::version;

fn watched(checks: &[Option<&str>]) -> Vec<UpdateStatus> {
    watched_from(false, checks)
}

fn watched_from(at_start: bool, checks: &[Option<&str>]) -> Vec<UpdateStatus> {
    let mut answers = checks.iter().map(|check| check.map(version));
    let mut rounds = checks.len();
    let sent = RefCell::new(Vec::new());
    let wait = || {
        rounds -= 1;
        rounds > 0
    };
    poll_releases(|| answers.next().flatten(), wait, at_start, &|status| {
        sent.borrow_mut().push(status)
    });
    sent.into_inner()
}

#[test]
fn the_watch_only_ever_offers_a_version() {
    let sent = watched(&[None, Some("9.0.0"), Some("9.0.0"), None, Some("9.1.0")]);
    let expected = ["9.0.0", "9.1.0"].map(|number| UpdateStatus::Available(number.into()));
    assert_eq!(sent, expected);
}

#[test]
fn the_same_version_is_offered_once() {
    assert_eq!(offer(Some(version("9.0.0")), Some(version("9.0.0"))), None);
    assert_eq!(offer(None, Some(version("9.0.0"))), None);
}

#[test]
fn the_first_answer_comes_without_waiting() {
    let sent = watched(&[Some("9.0.0")]);
    assert_eq!(sent, [UpdateStatus::Available("9.0.0".into())]);
}

#[test]
fn only_the_first_answer_of_a_plain_start_installs_at_once() {
    let sent = watched_from(true, &[Some("9.0.0"), Some("9.1.0")]);
    let expected = [
        UpdateStatus::AvailableAtStart("9.0.0".into()),
        UpdateStatus::Available("9.1.0".into()),
    ];
    assert_eq!(sent, expected);
    let late = watched_from(true, &[None, Some("9.0.0")]);
    assert_eq!(late, [UpdateStatus::Available("9.0.0".into())]);
}
