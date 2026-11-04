use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

use trame::session::pty::default_shell;
use trame::session::{Session, SessionEvent, SessionId, SessionOptions};
use trame::term::{DEFAULT_HISTORY, ReportedColors};

use crate::pty::{COLS, ROWS, TIMEOUT, screen_text};

pub(crate) fn start_shell(transcript: &[u8]) -> (Session, Receiver<SessionEvent>) {
    let shell = default_shell();
    let options = SessionOptions {
        shell: &shell,
        cwd: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        rows: ROWS,
        cols: COLS,
        history: DEFAULT_HISTORY,
        colors: ReportedColors::default(),
        transcript,
    };
    let (sender, events) = mpsc::channel();
    let notify = Arc::new(move |event: SessionEvent| {
        let _ = sender.send(event);
    });
    let session = Session::spawn(SessionId(1), &options, notify).expect("shell should start");
    (session, events)
}

pub(crate) fn wait_for(session: &Session, events: &Receiver<SessionEvent>, needle: &str) -> bool {
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
