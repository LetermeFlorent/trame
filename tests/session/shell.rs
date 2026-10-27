use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};

use trame::session::pty::default_shell;
use trame::session::{Session, SessionEvent, SessionId, SessionOptions};
use trame::term::ReportedColors;

use crate::pty::{COLS, ROWS};

pub(crate) fn start_shell(transcript: &[u8]) -> (Session, Receiver<SessionEvent>) {
    let shell = default_shell();
    let options = SessionOptions {
        shell: &shell,
        cwd: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        rows: ROWS,
        cols: COLS,
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
