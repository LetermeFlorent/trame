use trame::term::Modes;

use crate::pty::{COMMAND, screen_text};
use crate::shell::{start_shell, wait_for};

const LEAVE_MOUSE_ON: &[u8] = b"sh -c 'printf \"\\033[?1003h\\033[?1006h\"'\r";

#[test]
fn mouse_reports_stop_once_the_program_that_asked_is_gone() {
    let (session, events) = start_shell(&[]);
    session.write(LEAVE_MOUSE_ON);
    session.write(COMMAND);
    assert!(
        wait_for(&session, &events, "trame-42"),
        "shell went quiet:\n{}",
        screen_text(&session.terminal())
    );
    let modes = session.terminal().screen().modes();
    assert!(!modes.intersects(Modes::MOUSE_ANY));
    assert!(!modes.contains(Modes::MOUSE_SGR));
}
