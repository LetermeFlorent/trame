use std::env;
use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

use windows_sys::Win32::System::Threading::GetCurrentProcessId;

use super::process::{
    Cleanup, Target, WAIT_LIMIT, conhost_child, is_alive, snapshot, target_of, wait_until,
};
use crate::desk::{Desk, DeskSetup, Grids, Placement};
use crate::host::persist::Slot;
use crate::preferences::Preferences;
use crate::session::pty::default_shell;
use crate::theme::Appearance;

pub(super) const MODE_VAR: &str = "TRAME_PTY_TEST_MODE";
pub(super) const INFO_VAR: &str = "TRAME_PTY_TEST_INFO";

pub(super) fn open_desk_tab() -> (Desk, Target, Target) {
    let mut desk = Desk::new(DeskSetup {
        preferences: Preferences::default(),
        slot: Slot::new(None, 0),
        notifier: Arc::new(|_| {}),
        appearance: Appearance::Dark,
        icon_dir: None,
    });
    let _ = desk.open(Placement::Tab);
    desk.start_queued(&Grids::default());
    let tab = desk.workspace.active_tab().expect("active tab");
    let shell_pid = desk
        .sessions
        .get(tab)
        .and_then(|session| session.shell_id())
        .expect("shell pid");
    let shell = target_of(shell_pid).expect("shell target");
    let own = unsafe { GetCurrentProcessId() };
    let conhost = conhost_child(own).expect("conhost child");
    (desk, shell, conhost)
}

pub(super) fn launch_gui_from_desk(desk: &mut Desk, shell: Target, info: &Path) -> Target {
    let vbs = info.with_extension("vbs");
    let marker = info.with_extension("mrk");
    let _ = fs::remove_file(&marker);
    let script = format!(
        "CreateObject(\"Scripting.FileSystemObject\").CreateTextFile(\"{}\", True).Close\r\nWScript.Sleep 30000\r\n",
        marker.display()
    );
    fs::write(&vbs, script).expect("write vbs");
    let system_root = env::var_os("SystemRoot").expect("SystemRoot");
    let wscript = Path::new(&system_root).join(r"System32\wscript.exe");
    let program_name = default_shell()
        .program
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    let line = if program_name == "cmd.exe" {
        format!(
            "start \"\" \"{}\" //B //Nologo \"{}\"\r",
            wscript.display(),
            vbs.display()
        )
    } else {
        format!(
            "& \"{}\" //B //Nologo \"{}\"\r",
            wscript.display(),
            vbs.display()
        )
    };
    let _ = desk.write_to_active(|_| line.into_bytes());
    assert!(
        wait_until(WAIT_LIMIT, || marker.exists()),
        "gui marker never appeared"
    );
    snapshot()
        .into_iter()
        .find(|(_, ppid, name)| *ppid == shell.pid && name == "wscript.exe")
        .and_then(|(pid, _, _)| target_of(pid))
        .expect("wscript child of shell")
}

pub(super) fn spawn_child(test_name: &str, mode: &str, info_path: &Path) -> Child {
    let exe = env::current_exe().expect("current exe");
    Command::new(exe)
        .args(["--exact", test_name, "--quiet"])
        .env(MODE_VAR, mode)
        .env(INFO_VAR, info_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child test")
}

pub(super) fn write_targets(path: &Path, targets: &[Target]) {
    let text = targets
        .iter()
        .map(|target| format!("{}:{}", target.pid, target.born))
        .collect::<Vec<_>>()
        .join(" ");
    fs::write(path, text).expect("write targets");
}

pub(super) fn register_targets(info: &Path, cleanup: &mut Cleanup) -> Vec<Target> {
    if !info.exists() {
        return Vec::new();
    }
    let targets: Vec<Target> = fs::read_to_string(info)
        .expect("read targets")
        .split_whitespace()
        .map(|token| {
            let (pid, born) = token.split_once(':').expect("pid:born");
            Target {
                pid: pid.parse().expect("pid"),
                born: born.parse().expect("born"),
            }
        })
        .collect();
    for &target in &targets {
        cleanup.push(target);
    }
    targets
}

pub(super) fn assert_pair_ended(shell: Target, conhost: Target, context: &str) {
    let ended = wait_until(WAIT_LIMIT, || !is_alive(shell) && !is_alive(conhost));
    assert!(
        ended,
        "{context}: shell={} conhost={}",
        is_alive(shell),
        is_alive(conhost)
    );
}
