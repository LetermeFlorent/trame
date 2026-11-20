use std::env;
use std::os::windows::io::AsRawHandle;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use windows_sys::Win32::System::Threading::GetCurrentProcessId;

use super::harness::{MODE_VAR, assert_pair_ended, register_targets, spawn_child, write_targets};
use super::process::{
    Cleanup, Target, WAIT_LIMIT, born_of, conhost_child, is_alive, kill_if_alive, target_of,
    wait_until,
};
use super::{info_path_arg, prepare_run};
use crate::session::pty::{PtyOptions, PtySize, default_shell, spawn};

fn spawn_mode(test_name: &str, mode: &str) -> (Child, Target) {
    let exe = env::current_exe().expect("current exe");
    let child = Command::new(exe)
        .args(["--exact", test_name, "--quiet"])
        .env(MODE_VAR, mode)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child");
    let pid = child.id();
    let born = born_of(child.as_raw_handle()).expect("child born");
    (child, Target { pid, born })
}

#[test]
fn stale_born_timestamp_does_not_match_or_kill_reused_pid() {
    let test = "session::pty::windows::tests::guard::stale_born_timestamp_does_not_match_or_kill_reused_pid";
    if env::var_os(MODE_VAR).is_some() {
        sleep(Duration::from_secs(30));
        return;
    }
    let (mut child, real) = spawn_mode(test, "sleep");
    let mut safety = Cleanup::new();
    safety.push(real);

    let older = Target {
        pid: real.pid,
        born: real.born.saturating_sub(1),
    };
    let newer = Target {
        pid: real.pid,
        born: real.born.saturating_add(1),
    };
    assert!(is_alive(real));
    assert!(!is_alive(older));
    assert!(!is_alive(newer));

    let mut wrong_cleanup = Cleanup::new();
    wrong_cleanup.push(older);
    wrong_cleanup.push(newer);
    drop(wrong_cleanup);
    assert!(is_alive(real));

    drop(safety);
    assert!(wait_until(WAIT_LIMIT, || !is_alive(real)));
    let _ = child.wait();
}

#[test]
fn cleanup_skips_dead_and_kills_live() {
    let test = "session::pty::windows::tests::guard::cleanup_skips_dead_and_kills_live";
    if let Ok(mode) = env::var(MODE_VAR) {
        if mode == "sleep" {
            sleep(Duration::from_secs(30));
        }
        return;
    }
    let (mut exited, zombie) = spawn_mode(test, "exit");
    let _ = exited.wait();
    assert!(!is_alive(zombie));
    kill_if_alive(zombie);

    let (mut sleeper, real) = spawn_mode(test, "sleep");
    let mut safety = Cleanup::new();
    safety.push(real);

    let mut skipped = Cleanup::new();
    skipped.push(real);
    skipped.push(zombie);
    skipped.mark_dead(&[real]);
    drop(skipped);
    assert!(is_alive(real));

    drop(safety);
    assert!(wait_until(WAIT_LIMIT, || !is_alive(real)));
    let _ = sleeper.wait();
}

#[test]
fn pty_drop_ends_shell_and_conhost() {
    let test = "session::pty::windows::tests::guard::pty_drop_ends_shell_and_conhost";
    if env::var_os(MODE_VAR).is_some() {
        let shell_cmd = default_shell();
        let spawned = spawn(&PtyOptions {
            shell: &shell_cmd,
            cwd: None,
            size: PtySize::new(24, 80),
        })
        .expect("pty spawn");
        let shell = target_of(spawned.pty.process_id()).expect("shell");
        let own = unsafe { GetCurrentProcessId() };
        let conhost = conhost_child(own).expect("conhost");
        write_targets(&info_path_arg(), &[shell, conhost]);
        drop(spawned);
        assert_pair_ended(shell, conhost, "pty drop");
        return;
    }
    let (info, mut cleanup) = prepare_run("pty_drop");
    let mut child = spawn_child(test, "pty_drop", &info);
    let status = child.wait().expect("wait child");
    let targets = register_targets(&info, &mut cleanup);
    assert!(status.success(), "child failed: {status}");
    cleanup.mark_dead(&targets);
}
