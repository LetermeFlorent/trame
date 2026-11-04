use std::io::Write;
use std::path::Path;

use trame::session::pty::{PtyOptions, PtySize, default_shell, spawn};
use trame::term::Terminal;

use crate::pty::{COLS, ROWS, pump_until, read_chunks, screen_text};

#[cfg(windows)]
const PRINT_DUMP: &[u8] = b"echo ('dump-[' + $env:TRAME_DUMP + ']')\r";
#[cfg(unix)]
const PRINT_DUMP: &[u8] = b"echo \"dump-[$TRAME_DUMP]\"\r";

#[test]
fn the_shell_environment_has_no_dump_variable() {
    let mut shell = default_shell();
    shell.env.push(("TRAME_DUMP".into(), "recorded".into()));
    let options = PtyOptions {
        shell: &shell,
        cwd: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        size: PtySize::new(ROWS, COLS),
    };
    let spawned = spawn(&options).expect("shell should start");
    let mut writer = spawned.writer;
    let chunks = read_chunks(spawned.reader);
    let mut terminal = Terminal::new(ROWS, COLS, 1_000);
    writer.write_all(PRINT_DUMP).unwrap();
    pump_until(&mut terminal, &chunks, &mut writer, |terminal| {
        let text = screen_text(terminal);
        text.contains("dump-[]") || text.contains("dump-[recorded]")
    });
    let text = screen_text(&terminal);
    assert!(
        text.contains("dump-[]"),
        "the shell saw the variable:\n{text}"
    );
}
