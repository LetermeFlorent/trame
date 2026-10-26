use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Instant;

use trame::session::pty::default_shell;
use trame::session::{Session, SessionEvent, SessionId, SessionOptions};
use trame::term::{Modes, ReportedColors};

use crate::pty::{COLS, COMMAND, ROWS, TIMEOUT, screen_text};

const LEAVE_MOUSE_ON: &[u8] = b"sh -c 'printf \"\\033[?1003h\\033[?1006h\"'\r";

#[test]
fn mouse_reports_stop_once_the_program_that_asked_is_gone() {
    let shell = default_shell();
    let options = SessionOptions {
        shell: &shell,
        cwd: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        rows: ROWS,
        cols: COLS,
        colors: ReportedColors::default(),
        transcript: &[],
    };
    let (sender, events) = mpsc::channel();
    let notify = Arc::new(move |event: SessionEvent| {
        let _ = sender.send(event);
    });
    let session = Session::spawn(SessionId(1), &options, notify).expect("shell should start");
    session.write(LEAVE_MOUSE_ON);
    session.write(COMMAND);
    let deadline = Instant::now() + TIMEOUT;
    while !screen_text(&session.terminal()).contains("trame-42") {
        let left = deadline.saturating_duration_since(Instant::now());
        assert!(
            events.recv_timeout(left).is_ok(),
            "shell went quiet:\n{}",
            screen_text(&session.terminal())
        );
        session.take_output();
    }
    let modes = session.terminal().screen().modes();
    assert!(!modes.intersects(Modes::MOUSE_ANY));
    assert!(!modes.contains(Modes::MOUSE_SGR));
}
