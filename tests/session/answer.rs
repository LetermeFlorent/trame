use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Instant;

use trame::preferences::SecretAnswer;
use trame::session::pty::default_shell;
use trame::session::{Session, SessionEvent, SessionId, SessionOptions};
use trame::term::ReportedColors;

use crate::pty::{COLS, ROWS, TIMEOUT, screen_text};

#[cfg(windows)]
const ASK: &[u8] = b"$typed = Read-Host -Prompt \"me@box's password\"; 'got-' + $typed\r";
#[cfg(unix)]
const ASK: &[u8] = b"printf \"me@box's password: \"; read typed; echo got-$typed\r";

#[test]
fn a_password_prompt_on_a_real_shell_gets_the_secret() {
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
    session.write(ASK);
    let mut answer = SecretAnswer::new("hunter2".to_owned(), Instant::now());
    let mut replies = 0;
    let deadline = Instant::now() + TIMEOUT;
    while !screen_text(&session.terminal()).contains("got-hunter2") {
        let left = deadline.saturating_duration_since(Instant::now());
        let waited = events.recv_timeout(left);
        session.take_output();
        let line = session.terminal().screen().cursor_line();
        if let Some(reply) = answer.reply(&line, Instant::now()) {
            session.write(reply);
            replies += 1;
        }
        assert!(
            waited.is_ok(),
            "no answer:\n{}",
            screen_text(&session.terminal())
        );
    }
    assert_eq!(replies, 1);
}
