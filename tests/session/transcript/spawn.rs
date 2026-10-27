use std::sync::mpsc::Receiver;
use std::time::Instant;

use trame::session::{Session, SessionEvent};

use crate::pty::{COMMAND, TIMEOUT, screen_text};
use crate::shell::start_shell;

const SAVED: &str = "saved-before-restart";

fn wait_for(session: &Session, events: &Receiver<SessionEvent>, needle: &str) -> bool {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        session.take_output();
        if screen_text(&session.terminal()).contains(needle) {
            return true;
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if events.recv_timeout(left).is_err() {
            return false;
        }
    }
}

#[test]
fn restored_text_stays_above_the_new_shell() {
    let transcript = format!(
        "{SAVED}
"
    );
    let (session, events) = start_shell(transcript.as_bytes());
    session.write(COMMAND);
    let printed = wait_for(&session, &events, "trame-42");
    let text = screen_text(&session.terminal());
    assert!(printed, "command output missing:\n{text}");
    let saved = text.find(SAVED);
    assert!(
        saved.is_some_and(|at| text.find("trame-42").is_some_and(|output| at < output)),
        "restored text lost:\n{text}"
    );
}
