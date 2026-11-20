use std::sync::Arc;

use trame::desk::{Desk, DeskSetup, Grids, Placement};
use trame::host::persist::{Role, Slot, load_from};
use trame::preferences::Preferences;
use trame::theme::Appearance;

use crate::sandbox::Sandbox;

fn desk_in(sandbox: &Sandbox) -> Desk {
    Desk::new(DeskSetup {
        preferences: Preferences::default(),
        slot: Slot::new(Some(sandbox.root.as_path()), 0),
        notifier: Arc::new(|_| {}),
        appearance: Appearance::Dark,
        icon_dir: None,
    })
}

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
fn an_ssh_tab_keeps_its_session_key_across_restart_even_without_text_recall() {
    let sandbox = Sandbox::new("desk-saved-ssh");
    let mut first = desk_in(&sandbox);
    let _ = first.store_preferences(Preferences {
        restore_text: false,
        keep_remote_tasks: true,
        ..Preferences::default()
    });
    let host = trame::preferences::SshHost::parse("me@box:2222").expect("valid host");
    let _ = first.open(Placement::Tab);
    let _ = first.open(Placement::Tab);
    first.start_queued(&Grids::default());
    let tabs = first.workspace.tabs();
    first.workspace.set_tab_host(tabs[1], Some(host));
    let _ = first.save_session(false, None);
    let path = first.saver.slot().session_path().expect("a session file");
    let snapshot = load_from(&path, &mut Role::Guest).expect("a saved session");
    let mut second = desk_in(&sandbox);
    let _ = second.store_preferences(snapshot.preferences.clone());
    assert!(second.restore(Some(&snapshot)));
    let restored = second.workspace.tabs();
    assert_eq!(restored.len(), 2);
    assert_eq!(second.transcripts.key(restored[0]), None);
    let expected = first.transcripts.key(tabs[1]).expect("ssh key");
    assert_eq!(second.transcripts.key(restored[1]), Some(expected));
}
