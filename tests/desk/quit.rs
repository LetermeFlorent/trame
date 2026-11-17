use trame::host::quit::{Quit, Restart};

use crate::support::desk;

#[test]
fn a_close_asks_to_quit_until_taken() {
    let mut desk = desk();
    assert!(!desk.quitting());
    desk.quit_with(Quit::Close);
    assert!(desk.quitting());
    assert_eq!(desk.take_quit(), Some(Quit::Close));
    assert!(!desk.quitting());
    assert_eq!(desk.take_quit(), None);
}

#[test]
fn a_restart_is_never_replaced_by_a_close() {
    let mut desk = desk();
    desk.quit_with(Quit::Restart(Restart::Again));
    desk.quit_with(Quit::Close);
    assert_eq!(desk.take_quit(), Some(Quit::Restart(Restart::Again)));
}

#[test]
fn a_restart_replaces_a_close() {
    let mut desk = desk();
    desk.quit_with(Quit::Close);
    desk.quit_with(Quit::Restart(Restart::Update));
    assert_eq!(desk.take_quit(), Some(Quit::Restart(Restart::Update)));
}
