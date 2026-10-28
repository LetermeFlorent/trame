use std::time::Instant;

use trame::preferences::{SecretAnswer, SshHost, SshLogin};

use crate::pty::{TIMEOUT, screen_text};
use crate::shell::start_shell;

#[cfg(windows)]
const ASK: &[u8] = b"$typed = Read-Host -Prompt \"me@box's password\"; 'got-' + $typed\r";
#[cfg(unix)]
const ASK: &[u8] = b"printf \"me@box's password: \"; read typed; echo got-$typed\r";

#[test]
fn a_password_prompt_on_a_real_shell_gets_the_secret() {
    let (session, events) = start_shell(&[]);
    session.write(ASK);
    let host = SshHost::parse("me@box").expect("a host");
    let login = SshLogin::of(&host, None);
    let mut answer = SecretAnswer::new("hunter2".to_owned(), login, Instant::now());
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
