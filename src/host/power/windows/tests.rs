mod shell;

use std::os::windows::io::AsRawHandle;

use windows_sys::Win32::System::Threading::{
    BELOW_NORMAL_PRIORITY_CLASS, GetCurrentProcess, GetPriorityClass, IDLE_PRIORITY_CLASS,
};

use super::{Family, eco, throttle};
use shell::{
    ECO, Orphan, SHELL_PINGS, SLOWED, Shell, class_of, first_child_of, state_of, unsupported,
};

const PINNED: &str = "a job pins the priority class of the processes started here";

#[test]
fn full_speed_opts_out_of_windows_efficiency_mode() {
    if unsupported() {
        return;
    }
    let process = unsafe { GetCurrentProcess() };
    throttle(process, eco(false), unsafe { GetPriorityClass(process) });
    assert_eq!(state_of(process), Some((ECO, 0)));
}

#[test]
fn a_hidden_quiet_window_slows_and_restores_only_its_own_shell_trees() {
    if unsupported() {
        return;
    }
    let quiet = Shell::start();
    let [flooding, rested] = [Shell::start(), Shell::start()];
    let before = [quiet.state(), flooding.state()];
    let mut busy_window = Family::default();
    busy_window.apply(false, &[(flooding.id(), false), (rested.id(), true)]);
    let mut hidden_window = Family::default();
    let own = [(quiet.id(), false)];
    hidden_window.apply(true, &own);
    let slowed = [quiet.state(), flooding.state(), rested.state()];
    assert_eq!(slowed, [SLOWED, before[1], SLOWED]);
    hidden_window.apply(false, &own);
    let back = [quiet.state(), flooding.state(), rested.state()];
    assert_eq!(back, [before[0], before[1], SLOWED]);
}

#[test]
fn a_restore_reaches_what_an_exited_shell_left_behind() {
    if unsupported() {
        return;
    }
    let mut shell = Shell::reading_commands();
    shell.run(&format!("ping -n {SHELL_PINGS} 127.0.0.1 >nul"));
    let orphan = Orphan(first_child_of(shell.id()));
    let before = orphan.state();
    let own = [(shell.id(), false)];
    let mut window = Family::default();
    window.apply(true, &own);
    assert_eq!(orphan.state(), SLOWED);
    drop(shell);
    window.apply(false, &own);
    assert_eq!(orphan.state(), before);
}

#[test]
fn a_recheck_restores_what_a_resting_shell_started_before_exiting() {
    if unsupported() || class_pinned() {
        return;
    }
    let mut shell = Shell::reading_commands();
    let own = [(shell.id(), true)];
    let mut window = Family::default();
    window.apply(false, &own);
    shell.run(&format!("ping -n {SHELL_PINGS} 127.0.0.1 >nul"));
    let late = Orphan(first_child_of(shell.id()));
    let class = || class_of(late.0.handle.as_raw_handle());
    assert_eq!(class(), IDLE_PRIORITY_CLASS);
    eprintln!(
        "DBG shell={} late={} late_born={}",
        own[0].0, late.0.id, late.0.born
    );
    drop(shell);
    eprintln!("DBG ---- second apply");
    window.apply(false, &own);
    assert_eq!(class(), BELOW_NORMAL_PRIORITY_CLASS);
}

#[test]
fn a_restore_gives_each_shell_back_the_power_state_it_had() {
    if unsupported() || class_pinned() {
        return;
    }
    let shells = [Shell::start(), Shell::choosing_efficiency_mode()];
    let power = || {
        shells
            .each_ref()
            .map(|shell| (shell.class(), shell.state()))
    };
    let before = power();
    let own = shells.each_ref().map(|shell| (shell.id(), false));
    let mut window = Family::default();
    window.apply(true, &own);
    assert_eq!(power(), [(IDLE_PRIORITY_CLASS, SLOWED); 2]);
    window.apply(false, &own);
    assert_eq!(power(), before);
}

#[test]
fn a_restore_also_speeds_up_what_a_slowed_shell_started_meanwhile() {
    if unsupported() || class_pinned() {
        return;
    }
    let mut shell = Shell::reading_commands();
    let before = shell.state();
    let own = [(shell.id(), false)];
    let mut window = Family::default();
    window.apply(true, &own);
    assert_eq!(shell.class(), IDLE_PRIORITY_CLASS);
    shell.run(&format!("ping -n {SHELL_PINGS} 127.0.0.1 >nul"));
    let late = Orphan(first_child_of(shell.id()));
    let class = || class_of(late.0.handle.as_raw_handle());
    assert_eq!(class(), IDLE_PRIORITY_CLASS);
    window.apply(false, &own);
    assert_eq!([shell.class(), class()], [BELOW_NORMAL_PRIORITY_CLASS; 2]);
    assert_eq!(late.state(), before);
}

#[test]
fn a_shell_that_chose_idle_after_the_first_slow_keeps_it() {
    if unsupported() || class_pinned() {
        return;
    }
    let first = Shell::start();
    let mut window = Family::default();
    window.apply(true, &[(first.id(), false)]);
    let chose_idle = Shell::at(IDLE_PRIORITY_CLASS);
    let own = [(first.id(), false), (chose_idle.id(), false)];
    window.apply(true, &own);
    assert_eq!(chose_idle.state(), SLOWED);
    window.apply(false, &own);
    let back = [first.class(), chose_idle.class()];
    assert_eq!(back, [BELOW_NORMAL_PRIORITY_CLASS, IDLE_PRIORITY_CLASS]);
}

fn class_pinned() -> bool {
    let pinned = [IDLE_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS]
        .into_iter()
        .any(|class| Shell::at(class).class() != class);
    if pinned {
        assert!(std::env::var_os("CI").is_none(), "{PINNED}");
        eprintln!("skipped: {PINNED}");
    }
    pinned
}
