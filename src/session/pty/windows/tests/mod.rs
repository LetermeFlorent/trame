mod guard;
mod harness;
mod process;

use std::env;
use std::path::PathBuf;
use std::process as std_process;
use std::sync::atomic::{AtomicU32, Ordering};

use harness::{
    INFO_VAR, MODE_VAR, assert_pair_ended, launch_gui_from_desk, open_desk_tab, register_targets,
    spawn_child, write_targets,
};
use process::{Cleanup, conhost_child, is_alive, target_of};
use windows_sys::Win32::System::Threading::GetCurrentProcessId;

static NEXT_TAG: AtomicU32 = AtomicU32::new(1);

fn prepare_run(tag: &str) -> (PathBuf, Cleanup) {
    let seq = NEXT_TAG.fetch_add(1, Ordering::Relaxed);
    let info = env::temp_dir().join(format!("trame_pty_{tag}_{}_{seq}.txt", std_process::id()));
    let mut cleanup = Cleanup::new();
    cleanup.push_path(info.clone());
    (info, cleanup)
}

fn info_path_arg() -> PathBuf {
    PathBuf::from(env::var_os(INFO_VAR).expect("info path"))
}

#[test]
fn shell_and_conhost_end_when_desk_is_dropped() {
    let test = "session::pty::windows::tests::shell_and_conhost_end_when_desk_is_dropped";
    if env::var_os(MODE_VAR).is_some() {
        let (desk, shell, conhost) = open_desk_tab();
        write_targets(&info_path_arg(), &[shell, conhost]);
        assert!(is_alive(shell) && is_alive(conhost));
        drop(desk);
        assert_pair_ended(shell, conhost, "Desk drop");
        return;
    }
    let (info, mut cleanup) = prepare_run("drop_live");
    let mut child = spawn_child(test, "drop_live", &info);
    let status = child.wait().expect("wait child");
    let targets = register_targets(&info, &mut cleanup);
    assert!(status.success(), "child reported leak: {status}");
    cleanup.mark_dead(&targets);
}

#[test]
fn shell_and_conhost_end_when_process_exits_after_dropping_desk() {
    let test = "session::pty::windows::tests::shell_and_conhost_end_when_process_exits_after_dropping_desk";
    if env::var_os(MODE_VAR).is_some() {
        let (desk, shell, conhost) = open_desk_tab();
        write_targets(&info_path_arg(), &[shell, conhost]);
        drop(desk);
        std_process::exit(0);
    }
    let (info, mut cleanup) = prepare_run("drop_exit");
    let mut child = spawn_child(test, "drop_exit", &info);
    let status = child.wait().expect("wait child");
    let targets = register_targets(&info, &mut cleanup);
    assert!(status.success(), "child failed: {status}");
    let [shell, conhost] = targets[..] else {
        panic!("expected 2 targets");
    };
    assert_pair_ended(shell, conhost, "drop_exit");
    cleanup.mark_dead(&targets);
}

#[test]
fn shell_and_conhost_end_when_process_crashes_immediately() {
    let test =
        "session::pty::windows::tests::shell_and_conhost_end_when_process_crashes_immediately";
    if env::var_os(MODE_VAR).is_some() {
        let shell_cmd = crate::session::pty::default_shell();
        let spawned = crate::session::pty::spawn(&crate::session::pty::PtyOptions {
            shell: &shell_cmd,
            cwd: None,
            size: crate::session::pty::PtySize::new(24, 80),
        })
        .expect("pty spawn");
        let shell = target_of(spawned.pty.process_id()).expect("shell target");
        let own = unsafe { GetCurrentProcessId() };
        let conhost = conhost_child(own).expect("conhost child");
        write_targets(&info_path_arg(), &[shell, conhost]);
        std::mem::forget(spawned);
        std_process::abort();
    }
    let (info, mut cleanup) = prepare_run("crash_fast");
    let mut child = spawn_child(test, "crash_fast", &info);
    let status = child.wait().expect("wait child");
    let targets = register_targets(&info, &mut cleanup);
    assert!(!status.success());
    let [shell, conhost] = targets[..] else {
        panic!("expected 2 targets");
    };
    assert_pair_ended(shell, conhost, "crash_fast");
    cleanup.mark_dead(&targets);
}

#[test]
fn gui_app_started_from_shell_keeps_running_on_crash_and_desk_drop() {
    let test = "session::pty::windows::tests::gui_app_started_from_shell_keeps_running_on_crash_and_desk_drop";
    if let Ok(mode) = env::var(MODE_VAR) {
        let info = info_path_arg();
        let (mut desk, shell, conhost) = open_desk_tab();
        write_targets(&info, &[shell, conhost]);
        let gui = launch_gui_from_desk(&mut desk, shell, &info);
        write_targets(&info, &[shell, conhost, gui]);
        if mode == "gui_crash" {
            std::mem::forget(desk);
            std_process::abort();
        }
        drop(desk);
        assert_pair_ended(shell, conhost, "gui_drop child");
        assert!(is_alive(gui));
        return;
    }
    for mode in ["gui_drop", "gui_crash"] {
        let (info, mut cleanup) = prepare_run(mode);
        cleanup.push_path(info.with_extension("vbs"));
        cleanup.push_path(info.with_extension("mrk"));
        let mut child = spawn_child(test, mode, &info);
        let status = child.wait().expect("wait child");
        let targets = register_targets(&info, &mut cleanup);
        let expect_ok = mode == "gui_drop";
        assert_eq!(status.success(), expect_ok, "status: {status}");
        let [shell, conhost, gui] = targets[..] else {
            panic!("expected 3 targets");
        };
        assert_pair_ended(shell, conhost, mode);
        cleanup.mark_dead(&[shell, conhost]);
        assert!(is_alive(gui), "{mode}: gui exited prematurely");
    }
}
