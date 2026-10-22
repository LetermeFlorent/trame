use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use trame::session::pty::{PtyOptions, PtySize, default_shell, spawn};
use trame::term::Terminal;

const ROWS: usize = 24;
const COLS: usize = 100;
const TIMEOUT: Duration = Duration::from_secs(40);

#[cfg(windows)]
const COMMAND: &[u8] = b"echo ('trame-' + 6*7)\r";
#[cfg(unix)]
const COMMAND: &[u8] = b"echo trame-$((6*7))\r";

fn screen_text(terminal: &Terminal) -> String {
    let screen = terminal.screen();
    (0..screen.rows())
        .map(|row| screen.grid().row(row).text())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_chunks(mut reader: std::fs::File) -> Receiver<Vec<u8>> {
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

fn pump_until(
    terminal: &mut Terminal,
    chunks: &Receiver<Vec<u8>>,
    writer: &mut std::fs::File,
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
        if !responses.is_empty() {
            let _ = writer.write_all(&responses);
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
    let options = PtyOptions {
        shell: &shell,
        cwd: Some(cwd),
        size: PtySize::new(ROWS, COLS),
    };
    let spawned = spawn(&options).expect("shell should start");
    let mut writer = spawned.writer;
    let chunks = read_chunks(spawned.reader);
    let mut terminal = Terminal::new(ROWS, COLS, 1_000);

    writer.write_all(COMMAND).unwrap();
    let printed = pump_until(&mut terminal, &chunks, &mut writer, |terminal| {
        screen_text(terminal).contains("trame-42")
    });
    assert!(
        printed,
        "command output missing:\n{}",
        screen_text(&terminal)
    );

    let reported = terminal.screen().cwd().map(str::to_owned).or_else(|| {
        spawned
            .pty
            .cwd()
            .map(|path| path.to_string_lossy().into_owned())
    });
    assert!(
        reported.as_deref().is_some_and(|path| same_path(path, cwd)),
        "cwd not reported: {reported:?}"
    );

    spawned.pty.resize(PtySize::new(30, 80)).unwrap();
    writer.write_all(b"exit\r").unwrap();
    let (sender, receiver) = mpsc::channel();
    let child = spawned.child;
    thread::spawn(move || sender.send(child.wait()));
    let status = receiver.recv_timeout(TIMEOUT).expect("shell should exit");
    assert_eq!(status, Some(0));
    drop(spawned.pty);
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
