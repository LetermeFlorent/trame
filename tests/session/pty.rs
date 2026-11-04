use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use trame::session::pty::{Child, Pty, PtyOptions, PtySize, ShellCommand, default_shell, spawn};
use trame::term::Terminal;

pub(crate) const ROWS: usize = 24;
pub(crate) const COLS: usize = 100;
pub(crate) const TIMEOUT: Duration = Duration::from_secs(40);

#[cfg(windows)]
pub(crate) const COMMAND: &[u8] = b"echo ('trame-' + 6*7)\r";
#[cfg(unix)]
pub(crate) const COMMAND: &[u8] = b"echo trame-$((6*7))\r";

pub(crate) fn screen_text(terminal: &Terminal) -> String {
    let screen = terminal.screen();
    (0..screen.rows())
        .map(|row| screen.grid().row(row).text())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn read_chunks(mut reader: File) -> Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = [0; 4096];
        while let Ok(read) = reader.read(&mut buffer) {
            if read == 0 || sender.send(buffer[..read].to_vec()).is_err() {
                break;
            }
        }
    });
    receiver
}

pub(crate) struct Opened {
    pub(crate) pty: Pty,
    pub(crate) child: Child,
    pub(crate) writer: File,
    pub(crate) chunks: Receiver<Vec<u8>>,
    pub(crate) terminal: Terminal,
}

pub(crate) fn open_shell(shell: &ShellCommand, cwd: &Path) -> Opened {
    let options = PtyOptions {
        shell,
        cwd: Some(cwd),
        size: PtySize::new(ROWS, COLS),
    };
    let spawned = spawn(&options).expect("shell should start");
    Opened {
        pty: spawned.pty,
        child: spawned.child,
        writer: spawned.writer,
        chunks: read_chunks(spawned.reader),
        terminal: Terminal::new(ROWS, COLS, 1_000),
    }
}

pub(crate) fn pump_until(
    terminal: &mut Terminal,
    chunks: &Receiver<Vec<u8>>,
    writer: &mut File,
    done: impl Fn(&Terminal) -> bool,
) -> bool {
    let deadline = Instant::now() + TIMEOUT;
    while !done(terminal) {
        let left = deadline.saturating_duration_since(Instant::now());
        let Ok(chunk) = chunks.recv_timeout(left) else {
            return false;
        };
        terminal.feed(&chunk);
        let responses = terminal.screen_mut().take_responses();
        if !responses.is_empty() && writer.write_all(&responses).is_err() {
            return false;
        }
    }
    true
}

fn same_path(reported: &str, expected: &Path) -> bool {
    let normalize = |path: &str| path.trim_end_matches(['\\', '/']).to_lowercase();
    normalize(reported) == normalize(&expected.to_string_lossy())
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

    let reported = terminal
        .screen()
        .cwd()
        .map(str::to_owned)
        .or_else(|| pty.cwd().map(|path| path.to_string_lossy().into_owned()));
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
