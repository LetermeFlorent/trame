use trame::desk::{Grids, Placement};
use trame::host::persist::{Role, load_from};

use crate::sandbox::Sandbox;
use crate::support::{desk, desk_in};

#[test]
fn a_saved_session_comes_back() {
    let sandbox = Sandbox::new("desk-saved");
    let mut first = desk_in(&sandbox);
    let _ = first.open(Placement::Tab);
    let _ = first.open(Placement::Tab);
    first.start_queued(&Grids::default());
    assert_eq!(first.workspace.tabs().len(), 2);
    let _ = first.save_session(false, None);
    let path = first.saver.slot().session_path().expect("a session file");
    let snapshot = load_from(&path, &mut Role::Guest).expect("a saved session");
    let mut second = desk_in(&sandbox);
    assert!(second.restore(Some(&snapshot)));
    let tabs = second.workspace.tabs();
    assert_eq!(tabs.len(), 2);
    assert!(tabs.iter().all(|id| second.sessions.is_waiting(*id)));
    let mut grids = Grids::default();
    for id in &tabs {
        grids.add_pane(&[*id], Some(*id), (24, 80));
    }
    assert!(second.start_shown(&grids));
    assert!(tabs.iter().all(|id| second.sessions.get(*id).is_some()));
    assert!(second.settle_restore());
}

#[test]
fn nothing_to_restore() {
    let sandbox = Sandbox::new("desk-unsaved");
    let mut desk = desk_in(&sandbox);
    assert!(!desk.restore(None));
}

#[test]
fn a_desk_that_owns_no_slot_restores_nothing() {
    let sandbox = Sandbox::new("desk-guest");
    let mut first = desk_in(&sandbox);
    let _ = first.open(Placement::Tab);
    let _ = first.save_session(false, None);
    let path = first.saver.slot().session_path().expect("a session file");
    let snapshot = load_from(&path, &mut Role::Guest).expect("a saved session");
    let mut guest = desk();
    assert!(!guest.restore(Some(&snapshot)));
    assert!(guest.workspace.is_empty());
}

#[test]
fn a_restored_desk_opens_nothing_more() {
    let mut desk = desk();
    assert!(desk.open_first(true, None).is_empty());
    assert!(!desk.settle_start());
}

#[test]
fn a_fresh_start_opens_a_first_instance() {
    let mut desk = desk();
    assert!(!desk.open_first(false, None).is_empty());
    desk.start_queued(&Grids::default());
    assert_eq!(desk.workspace.instances.len(), 1);
    assert!(desk.settle_start());
}
