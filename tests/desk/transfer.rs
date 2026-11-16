use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use trame::desk::{Desk, DeskSetup, Effect, Grids, Placement};
use trame::host::persist::Slot;
use trame::preferences::Preferences;
use trame::session::{SessionEvent, SessionId};
use trame::theme::Appearance;

use crate::tabs::opened;

const TIMEOUT: Duration = Duration::from_secs(40);
const PRINT: &[u8] = b"echo trame-output\r";

fn listening() -> (Desk, Receiver<SessionEvent>) {
    let (sender, events) = mpsc::channel();
    let mut desk = Desk::new(DeskSetup {
        preferences: Preferences::default(),
        slot: Slot::new(None, 0),
        notifier: Arc::new(move |event| {
            let _ = sender.send(event);
        }),
        appearance: Appearance::Dark,
        icon_dir: None,
    });
    let _ = desk.open(Placement::Tab);
    desk.start_queued(&Grids::default());
    (desk, events)
}

fn with_two_tabs() -> (Desk, SessionId, SessionId) {
    let (mut desk, first) = opened();
    let _ = desk.open(Placement::Tab);
    desk.start_queued(&Grids::default());
    let second = desk.workspace.active_tab().expect("a second tab");
    (desk, first, second)
}

fn wakes(events: &Receiver<SessionEvent>, id: SessionId) -> bool {
    let give_up = Instant::now() + TIMEOUT;
    while let Some(left) = give_up.checked_duration_since(Instant::now()) {
        match events.recv_timeout(left) {
            Ok(SessionEvent::Output(woken)) if woken == id => return true,
            Ok(_) => {}
            Err(_) => return false,
        }
    }
    false
}

#[test]
fn a_torn_tab_keeps_its_shell_in_another_desk() {
    let (mut source, first, second) = with_two_tabs();
    let _ = source.close_tab(false, first);
    assert!(!source.can_tear(second));
    let (mut source, _, second) = with_two_tabs();
    assert!(source.can_tear(second));
    let torn = source.tear_tab(second);
    assert_eq!(torn.effects.as_slice(), [Effect::Animate, Effect::Changed]);
    assert!(source.sessions.get(second).is_none());
    assert!(source.workspace.locate(second).is_none());
    let (mut target, events) = listening();
    let id = target.receive_tab(torn.value.expect("a torn tab"), None);
    assert!(target.workspace.locate(id).is_some());
    target.sessions.get(id).expect("the shell").write(PRINT);
    assert!(wakes(&events, id));
}

#[test]
fn a_given_instance_moves_all_its_tabs() {
    let (mut source, first, second) = with_two_tabs();
    let _ = source.open(Placement::Instance);
    source.start_queued(&Grids::default());
    let instance = source.workspace.instances[0].id;
    let moved = source.give_instance(instance);
    assert_eq!(moved.effects.as_slice(), [Effect::Animate, Effect::Changed]);
    assert!(source.sessions.get(first).is_none());
    assert!(source.sessions.get(second).is_none());
    let (mut target, _) = listening();
    let effects = target.receive_instance(moved.value.expect("an instance"), None);
    assert_eq!(effects.as_slice(), target.welcome().as_slice());
    assert_eq!(target.workspace.instances.len(), 2);
    let tabs = target.workspace.instances[1].tabs();
    assert_eq!(tabs.len(), 2);
    assert!(tabs.iter().all(|tab| target.sessions.get(*tab).is_some()));
}
