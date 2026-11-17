use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use trame::desk::{Desk, Effect, Grids, Placement, Seen};
use trame::session::{SessionEvent, SessionId};

use crate::support::desk_with;

const TIMEOUT: Duration = Duration::from_secs(40);
const COPY: &[u8] = b"printf '\\033]52;c;aGVsbG8=\\007'\r";
const CALL: &[u8] = b"printf '\\033]9;done\\007'\r";
const DROPPED: &[u8] = b"printf '\\033]133;C\\007\\033]133;D;255\\007'\r";
const UNSEEN: Seen = Seen {
    focused: false,
    showing: false,
};

fn started() -> (Desk, SessionId, Receiver<SessionEvent>) {
    let (sender, events) = mpsc::channel();
    let mut desk = desk_with(Arc::new(move |event| {
        let _ = sender.send(event);
    }));
    let _ = desk.open(Placement::Tab);
    desk.start_queued(&Grids::default());
    let id = desk.workspace.active_tab().expect("a tab");
    (desk, id, events)
}

fn effects_until(
    desk: &mut Desk,
    events: &Receiver<SessionEvent>,
    seen: Seen,
    wanted: impl Fn(&Effect) -> bool,
) -> Vec<Effect> {
    let give_up = Instant::now() + TIMEOUT;
    let mut all = Vec::new();
    while Instant::now() < give_up {
        let Ok(SessionEvent::Output(id)) = events.recv_timeout(TIMEOUT) else {
            break;
        };
        all.extend(desk.output(id, seen));
        if all.iter().any(&wanted) {
            break;
        }
    }
    all
}

#[test]
fn a_focused_tab_hands_its_copy_to_the_clipboard() {
    let (mut desk, id, events) = started();
    desk.sessions.get(id).expect("the shell").write(COPY);
    let seen = Seen {
        focused: true,
        showing: true,
    };
    let copied = Effect::Clipboard("hello".into());
    let effects = effects_until(&mut desk, &events, seen, |effect| *effect == copied);
    assert!(effects.contains(&copied));
}

#[test]
fn an_unseen_tab_calls_for_attention_without_copying() {
    let (mut desk, id, events) = started();
    assert!(desk.preferences.notify_waiting);
    let shell = desk.sessions.get(id).expect("the shell");
    shell.write(COPY);
    shell.write(CALL);
    let called = |effect: &Effect| matches!(effect, Effect::Notify { body, .. } if body == "done");
    let effects = effects_until(&mut desk, &events, UNSEEN, called);
    assert!(effects.iter().any(called));
    let copied = |effect: &Effect| matches!(effect, Effect::Clipboard(_));
    assert!(!effects.iter().any(copied));
}

#[test]
fn a_dropped_remote_link_relayouts() {
    let (mut desk, id, events) = started();
    desk.sessions.watch_remote(id, "127.0.0.1".into(), 9);
    desk.sessions.watch_remote(id, "127.0.0.1".into(), 9);
    desk.sessions.get(id).expect("the shell").write(DROPPED);
    let relayout = |effect: &Effect| *effect == Effect::Relayout;
    let effects = effects_until(&mut desk, &events, UNSEEN, relayout);
    assert!(effects.iter().any(relayout));
    assert!(desk.sessions.offline(id));
}
