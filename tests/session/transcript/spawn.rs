use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

use trame::session::pty::default_shell;
use trame::session::{Session, SessionEvent, SessionId, SessionOptions};
use trame::term::ReportedColors;

use crate::pty::{COLS, COMMAND, ROWS, TIMEOUT, screen_text};

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
    let shell = default_shell();
    let transcript = format!("{SAVED}\r\n");
    let options = SessionOptions {
        shell: &shell,
        cwd: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        rows: ROWS,
        cols: COLS,
        colors: ReportedColors::default(),
        transcript: transcript.as_bytes(),
    };
    let (sender, events) = mpsc::channel();
    let notify = Arc::new(move |event: SessionEvent| {
        let _ = sender.send(event);
    });
    let session = Session::spawn(SessionId(1), &options, notify).expect("shell should start");
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
