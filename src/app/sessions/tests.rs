use std::net::TcpListener;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use trame::session::{SessionEvent, SessionId};
use trame::term::ReportedColors;

use super::remotes::RETRY;
use super::{Sessions, ShellStart};

const TIMEOUT: Duration = Duration::from_secs(40);
const PRINT: &[u8] = b"echo trame-output\r";

fn started() -> (Sessions, SessionId, Receiver<SessionEvent>) {
    let (sender, events) = mpsc::channel();
    let mut sessions = Sessions::with_notifier(Arc::new(move |event| {
        let _ = sender.send(event);
    }));
    let id = sessions.reserve();
    let start = ShellStart {
        cwd: None,
        size: (24, 80),
        colors: ReportedColors::default(),
        transcript: &[],
    };
    sessions.spawn(id, &start).expect("a shell starts");
    (sessions, id, events)
}

fn wakes(events: &Receiver<SessionEvent>, id: SessionId) -> bool {
    events.recv_timeout(TIMEOUT) == Ok(SessionEvent::Output(id))
}

#[test]
fn output_keeps_waking_without_a_draw() {
    let (mut sessions, id, events) = started();
    assert!(wakes(&events, id));
    sessions.output(id);
    sessions.get(id).expect("the shell").write(PRINT);
    assert!(wakes(&events, id));
}

#[test]
fn a_handed_over_shell_wakes_its_new_window_under_its_new_id() {
    let (mut sessions, id, events) = started();
    assert!(wakes(&events, id));
    let (sender, moved) = mpsc::channel();
    let mut other = Sessions::with_notifier(Arc::new(move |event| {
        let _ = sender.send(event);
    }));
    other.reserve();
    let new = other.take_in(sessions.hand_off(id));
    assert_ne!(new, id);
    assert!(sessions.get(id).is_none());
    other.get(new).expect("the shell").write(PRINT);
    assert!(wakes(&moved, new));
}

fn redialed(sessions: &mut Sessions) -> Vec<SessionId> {
    let later = Instant::now() + RETRY;
    let give_up = Instant::now() + TIMEOUT;
    loop {
        let due = sessions.redial_due(later);
        if !due.is_empty() || Instant::now() > give_up {
            return due;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_dropped_ssh_link_waits_for_its_server_then_redials() {
    let server = TcpListener::bind("127.0.0.1:0").expect("a local port");
    let port = server.local_addr().expect("its address").port();
    let mut sessions = Sessions::with_notifier(Arc::new(|_| {}));
    let id = sessions.reserve();
    sessions.watch_remote(id, "127.0.0.1".into(), port);
    assert!(!sessions.ended(id, 255));
    assert!(!sessions.offline(id));
    sessions.watch_remote(id, "127.0.0.1".into(), port);
    sessions.watch_remote(id, "127.0.0.1".into(), port);
    assert!(!sessions.ended(id, 0));
    sessions.watch_remote(id, "127.0.0.1".into(), port);
    sessions.watch_remote(id, "127.0.0.1".into(), port);
    assert!(sessions.ended(id, 255));
    assert!(sessions.offline(id));
    assert!(sessions.remote_deadline(Instant::now()).is_some());
    assert!(sessions.redial_due(Instant::now()).is_empty());
    assert_eq!(redialed(&mut sessions), vec![id]);
    sessions.forget_remote(id);
    assert!(!sessions.offline(id));
    assert!(sessions.remote_deadline(Instant::now()).is_none());
}
