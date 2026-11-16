use trame::desk::{Desk, Effect, Grids, Placement};
use trame::host::quit::Quit;
use trame::session::SessionId;

use crate::support::desk;

const STARTS: [Effect; 3] = [Effect::Relayout, Effect::StartQueued, Effect::Changed];

pub fn opened() -> (Desk, SessionId) {
    let mut desk = desk();
    assert_eq!(desk.open(Placement::Tab).as_slice(), STARTS);
    desk.start_queued(&Grids::default());
    let id = desk.workspace.active_tab().expect("a tab");
    assert!(desk.sessions.get(id).is_some());
    (desk, id)
}

#[test]
fn a_new_tab_starts_in_the_size_of_its_pane() {
    let (mut desk, _) = opened();
    assert_eq!(desk.open(Placement::Tab).as_slice(), STARTS);
    assert_eq!(desk.workspace.tabs().len(), 2);
    let id = desk.workspace.active_tab().expect("the new tab");
    assert!(desk.sessions.get(id).is_none());
    let mut grids = Grids::default();
    grids.add_pane(&desk.workspace.tabs(), Some(id), (30, 100));
    desk.start_queued(&grids);
    let session = desk.sessions.get(id).expect("the new shell");
    assert_eq!(session.size(), (30, 100));
    let terminal = session.terminal();
    assert_eq!(
        (terminal.screen().rows(), terminal.screen().cols()),
        (30, 100)
    );
}

#[test]
fn closing_the_last_tab_opens_a_fresh_one() {
    let (mut desk, old) = opened();
    let effects = desk.close_tab(false, old);
    assert_eq!(&effects.as_slice()[..3], STARTS);
    let tabs = desk.workspace.tabs();
    assert_eq!(tabs.len(), 1);
    assert_ne!(tabs[0], old);
    assert!(desk.sessions.get(old).is_none());
    desk.start_queued(&Grids::default());
    assert!(desk.sessions.get(tabs[0]).is_some());
    assert!(!desk.quitting());
}

#[test]
fn closing_the_last_tab_while_exiting_quits() {
    let (mut desk, old) = opened();
    let _ = desk.close_tab(true, old);
    assert!(desk.workspace.is_empty());
    assert_eq!(desk.take_quit(), Some(Quit::Close));
}

#[test]
fn a_fresh_tab_that_exits_at_once_stays_open() {
    let (mut desk, old) = opened();
    let _ = desk.shell_exited(false, old);
    desk.start_queued(&Grids::default());
    let fresh = desk.workspace.active_tab().expect("a fresh tab");
    assert_ne!(fresh, old);
    let effects = desk.shell_exited(false, fresh);
    assert_eq!(effects.as_slice(), [Effect::Redraw]);
    assert_eq!(desk.workspace.tabs(), [fresh]);
}

#[test]
fn disconnecting_asks_for_a_new_layout() {
    let (mut desk, id) = opened();
    let effects = desk.disconnect(id);
    assert_eq!(effects.as_slice(), [Effect::Relayout, Effect::Redraw]);
}
