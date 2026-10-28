use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use trame::session::{SessionEvent, SessionId};
use trame::term::ReportedColors;

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
