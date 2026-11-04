use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use trame::session::pty::{Child, Pty, PtyOptions, PtySize, ShellCommand, spawn};
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
