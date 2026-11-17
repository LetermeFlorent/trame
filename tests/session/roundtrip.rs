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

#[cfg(windows)]
const STATUS_PROBE: &str = "$e = [string][char]27; $b = [string][char]7; [Console]::Write($e + ']7501;?' + $b); $got = ''; $end = [DateTime]::Now.AddSeconds(5); while ([DateTime]::Now -lt $end -and -not $got.EndsWith($b)) { if ([Console]::KeyAvailable) { $got += [Console]::ReadKey($true).KeyChar } else { Start-Sleep -Milliseconds 10 } }; [Console]::Write('reply:' + $got.Replace($e, '^').Replace($b, '!') + ':end')";

#[cfg(windows)]
const STATUS_REPLY: &str =
    "reply:^]7501;?:states=idle,working,done,blocked,error:kinds=permission,question,auth!:end";

#[cfg(windows)]
const PROBE: &[u8] = b"\x1b]7501;?";

#[cfg(windows)]
fn watch_probe(
    chunks: Receiver<Vec<u8>>,
) -> (
    Receiver<Vec<u8>>,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    use std::sync::atomic::Ordering;

    let seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = std::sync::Arc::clone(&seen);
    let (sender, forwarded) = mpsc::channel();
    thread::spawn(move || {
        let mut received = Vec::new();
        while let Ok(chunk) = chunks.recv() {
            received.extend_from_slice(&chunk);
            if received.windows(PROBE.len()).any(|window| window == PROBE) {
                flag.store(true, Ordering::SeqCst);
            }
            if sender.send(chunk).is_err() {
                break;
            }
        }
    });
    (forwarded, seen)
}

#[cfg(windows)]
fn joined_screen(terminal: &Terminal) -> String {
    screen_text(terminal).replace('\n', "")
}

#[cfg(windows)]
#[test]
fn a_program_behind_conpty_reads_the_program_status_reply() {
    let system_root = std::env::var_os("SystemRoot").expect("SystemRoot is set");
    let shell = trame::session::pty::ShellCommand {
        program: Path::new(&system_root).join(r"System32\WindowsPowerShell\v1.0\powershell.exe"),
        args: ["-NoProfile", "-Command", STATUS_PROBE]
            .into_iter()
            .map(std::ffi::OsString::from)
            .collect(),
        env: Vec::new(),
    };
    let Opened {
        pty: _pty,
        child: _child,
        mut writer,
        chunks,
        mut terminal,
    } = open_shell(&shell, Path::new(env!("CARGO_MANIFEST_DIR")));
    let (chunks, probe_seen) = watch_probe(chunks);
    let finished = pump_until(&mut terminal, &chunks, &mut writer, |terminal| {
        joined_screen(terminal).contains(":end")
    });
    let screen = joined_screen(&terminal);
    assert!(finished, "no output before the timeout:\n{screen}");
    if !probe_seen.load(std::sync::atomic::Ordering::SeqCst) {
        println!("skipped: conhost did not forward OSC 7501");
        return;
    }
    assert!(screen.contains(STATUS_REPLY), "reply lost:\n{screen}");
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
