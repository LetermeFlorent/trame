use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};

use trame::session::pty::default_shell;
use trame::session::{Session, SessionEvent, SessionId, SessionOptions};
use trame::term::ReportedColors;

use crate::pty::{COLS, ROWS};

pub(crate) fn start_shell(transcript: &[u8]) -> (Session, Receiver<SessionEvent>) {
    let shell = default_shell();
    let options = SessionOptions {
        shell: &shell,
        cwd: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        rows: ROWS,
        cols: COLS,
        colors: ReportedColors::default(),
        transcript,
    };
    let (sender, events) = mpsc::channel();
    let notify = Arc::new(move |event: SessionEvent| {
        let _ = sender.send(event);
    });
    let session = Session::spawn(SessionId(1), &options, notify).expect("shell should start");
    (session, events)
}

#[cfg(windows)]
#[test]
fn a_relative_path_entry_is_skipped() {
    use std::fs;
    use trame::session::pty::find_in_path;
    let program = "pwsh.exe";
    let tag = format!("trame-path-{}", std::process::id());
    let relative = Path::new("target").join(&tag);
    let absolute = std::env::temp_dir().join(&tag);
    for folder in [&relative, &absolute] {
        fs::create_dir_all(folder).expect("the folder");
        fs::write(folder.join(program), b"").expect("the program");
    }
    let path = std::env::join_paths([&relative, &absolute]).expect("a PATH value");
    let found = find_in_path(&path, program);
    for folder in [&relative, &absolute] {
        let _ = fs::remove_dir_all(folder);
    }
    assert_eq!(found, Some(absolute.join(program)));
}
