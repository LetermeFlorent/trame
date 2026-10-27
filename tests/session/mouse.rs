use std::time::Instant;

use trame::term::Modes;

use crate::pty::{COMMAND, TIMEOUT, screen_text};
use crate::shell::start_shell;

const LEAVE_MOUSE_ON: &[u8] = b"sh -c 'printf \"\\033[?1003h\\033[?1006h\"'\r";

#[test]
fn mouse_reports_stop_once_the_program_that_asked_is_gone() {
    let (session, events) = start_shell(&[]);
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
