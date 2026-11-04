use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Instant;

use trame::session::pty::{Pty, PtySize, default_shell};
use trame::term::Terminal;

use crate::pty::{COMMAND, Opened, TIMEOUT, open_shell, pump_until, screen_text};

fn same_path(reported: &str, expected: &Path) -> bool {
    let normalize = |path: &str| path.trim_end_matches(['\\', '/']).to_lowercase();
    normalize(reported) == normalize(&expected.to_string_lossy())
}

fn reported_cwd(terminal: &Terminal, pty: &Pty) -> Option<String> {
    terminal
        .screen()
        .cwd()
        .map(str::to_owned)
        .or_else(|| pty.cwd().map(|path| path.to_string_lossy().into_owned()))
}

#[test]
fn shell_runs_a_command_and_exits() {
    let shell = default_shell();
    let cwd = Path::new(env!("CARGO_MANIFEST_DIR"));
    let Opened {
        pty,
        child,
        mut writer,
        chunks,
        mut terminal,
    } = open_shell(&shell, cwd);

    writer.write_all(COMMAND).unwrap();
    let printed = pump_until(&mut terminal, &chunks, &mut writer, |terminal| {
        screen_text(terminal).contains("trame-42")
    });
    assert!(
        printed,
        "command output missing:\n{}",
        screen_text(&terminal)
    );

    let reported = reported_cwd(&terminal, &pty);
    assert!(
        reported.as_deref().is_some_and(|path| same_path(path, cwd)),
        "cwd not reported: {reported:?}"
    );

    pty.resize(PtySize::new(30, 80)).unwrap();
    writer.write_all(b"exit\r").unwrap();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || sender.send(child.wait()));
    let status = receiver.recv_timeout(TIMEOUT).expect("shell should exit");
    assert_eq!(status, Some(0));
    drop(pty);
    assert!(output_ends(&chunks), "reader never reached end of output");
}

fn output_ends(chunks: &Receiver<Vec<u8>>) -> bool {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        match chunks.recv_timeout(left) {
            Ok(_) => continue,
            Err(RecvTimeoutError::Disconnected) => return true,
            Err(RecvTimeoutError::Timeout) => return false,
        }
    }
}
