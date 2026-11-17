use trame::desk::Effect;
use trame::host::pending::Offer;
use trame::host::quit::{Quit, Restart};

use crate::support::desk;

fn offer() -> Offer {
    Offer {
        version: "9.9.9".to_owned(),
    }
}

#[test]
fn an_offer_is_kept_and_changes_the_window() {
    let mut desk = desk();
    let effects = desk.offer_update(offer());
    assert_eq!(effects.as_slice(), [Effect::Changed]);
    assert_eq!(desk.update.offer, Some(offer()));
}

#[test]
fn no_restart_for_an_update_without_an_offer() {
    let mut desk = desk();
    desk.restart_for_update();
    assert!(!desk.quitting());
}

#[test]
fn an_offered_update_restarts() {
    let mut desk = desk();
    let _ = desk.offer_update(offer());
    desk.restart_for_update();
    assert_eq!(desk.take_quit(), Some(Quit::Restart(Restart::Update)));
}

#[test]
fn a_failed_update_is_remembered() {
    let mut desk = desk();
    let effects = desk.update_failed();
    assert_eq!(effects.as_slice(), [Effect::Changed]);
    assert!(desk.update.failed);
}

#[test]
fn restarting_marks_the_desk() {
    let mut desk = desk();
    desk.mark_restarting();
    assert!(desk.update.restarting);
    assert!(!desk.saver.owns());
}
