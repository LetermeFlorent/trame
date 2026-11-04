use crate::pty::{COMMAND, screen_text};
use crate::shell::{start_shell, wait_for};

const SAVED: &str = "saved-before-restart";

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
