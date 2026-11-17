use trame::desk::{Closing, Confirm, Desk, Effect, Grids, Placement};
use trame::host::pending::Offer;
use trame::host::quit::{Quit, Restart};
use trame::locale::COUNT_SLOT;

use crate::tabs::opened;

fn two_tabs() -> Desk {
    let (mut desk, _) = opened();
    desk.preferences.confirm_close = true;
    let _ = desk.open(Placement::Tab);
    desk.start_queued(&Grids::default());
    desk
}

#[test]
fn closing_a_window_with_tabs_asks_first() {
    let mut desk = two_tabs();
    let effects = desk.ask_close(Closing::Window, false);
    assert_eq!(effects.as_slice(), [Effect::Changed]);
    assert!(matches!(desk.closing, Some(Closing::Window)));
    let question = desk.question().expect("a question");
    assert_eq!(question.confirm, Confirm::Close);
    let body = desk.strings.tabs_will_close.replace(COUNT_SLOT, "2");
    assert_eq!(question.body, body);
}

#[test]
fn a_cancelled_close_keeps_the_window() {
    let mut desk = two_tabs();
    let _ = desk.ask_close(Closing::Window, false);
    let effects = desk.settle_closing(false);
    assert_eq!(effects.as_slice(), [Effect::Changed]);
    assert!(desk.closing.is_none());
    assert!(!desk.quitting());
}

#[test]
fn a_confirmed_close_quits() {
    let mut desk = two_tabs();
    let _ = desk.ask_close(Closing::Window, false);
    let _ = desk.settle_closing(true);
    assert_eq!(desk.take_quit(), Some(Quit::Close));
}

#[test]
fn closing_while_loading_quits_at_once() {
    let mut desk = two_tabs();
    let _ = desk.ask_close(Closing::Window, true);
    assert!(desk.closing.is_none());
    assert_eq!(desk.take_quit(), Some(Quit::Close));
}

#[test]
fn an_update_without_an_offer_asks_nothing() {
    let mut desk = two_tabs();
    let _ = desk.ask_close(Closing::Update, false);
    assert!(desk.question().is_none());
}

#[test]
fn an_offered_update_asks_to_restart() {
    let mut desk = two_tabs();
    let _ = desk.offer_update(Offer {
        version: "9.9.9".to_owned(),
    });
    let _ = desk.ask_close(Closing::Update, false);
    let question = desk.question().expect("a question");
    assert_eq!(question.confirm, Confirm::Restart);
    let _ = desk.settle_closing(true);
    assert_eq!(desk.take_quit(), Some(Quit::Restart(Restart::Update)));
}

#[test]
fn an_instance_closes_at_once_without_confirmation() {
    let mut desk = two_tabs();
    desk.preferences.confirm_close = false;
    let _ = desk.open(Placement::Instance);
    desk.start_queued(&Grids::default());
    let first = desk.workspace.instances[0].id;
    let effects = desk.ask_close(Closing::Instance(first), false);
    assert_eq!(effects.as_slice(), [Effect::Animate, Effect::Changed]);
    assert!(desk.closing.is_none());
    assert_eq!(desk.workspace.instances.len(), 1);
    assert_ne!(desk.workspace.instances[0].id, first);
}
