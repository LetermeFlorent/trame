use std::io::Write;
use std::path::Path;

use trame::session::pty::default_shell;

use crate::pty::{Opened, open_shell, pump_until, screen_text};

#[cfg(windows)]
const PRINT_DUMP: &[u8] = b"echo ('dump-[' + $env:TRAME_DUMP + ']')\r";
#[cfg(unix)]
const PRINT_DUMP: &[u8] = b"echo \"dump-[$TRAME_DUMP]\"\r";

#[test]
fn the_shell_environment_has_no_dump_variable() {
    let mut shell = default_shell();
    shell.env.push(("TRAME_DUMP".into(), "recorded".into()));
    let Opened {
        pty: _pty,
        child: _child,
        mut writer,
        chunks,
        mut terminal,
    } = open_shell(&shell, Path::new(env!("CARGO_MANIFEST_DIR")));
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
