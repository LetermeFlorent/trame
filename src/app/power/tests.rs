mod idle_repaint;

use std::time::Instant;

use trame::session::SessionId;

use super::rest::{RECHECKS_RESTING_SHELLS, REST_AFTER};
use super::*;

fn tab(id: u64, shown: bool) -> Shown {
    Shown {
        id: SessionId(id),
        shell: 0,
        shown,
    }
}

#[test]
fn output_keeps_a_hidden_window_and_its_rested_tab_at_full_speed_until_quiet() {
    let mut power = Power::new([true, true]);
    power.set_tabs([tab(1, false)]);
    power.set_background(true);
    power.tick(Instant::now() + REST_AFTER);
    assert_eq!(power.mode, Mode::Family);
    assert!(power.rest.shells()[0].1);
    assert_eq!(power.deadline(), Some(power.walked + FAMILY_RECHECK));
    power.output(SessionId(1), false);
    let heard = Instant::now();
    assert_eq!(power.mode, Mode::Normal);
    assert!(!power.rest.shells()[0].1);
    assert!(power.hidden());
    let wake = power
        .deadline()
        .expect("a hidden window waits for quiet output");
    assert!((heard + REST_AFTER / 2..=heard + REST_AFTER).contains(&wake));
    power.tick(heard + REST_AFTER / 2);
    assert_eq!(power.mode, Mode::Normal);
    power.tick(heard + REST_AFTER);
    assert_eq!(power.mode, Mode::Family);
}

#[test]
fn coming_back_to_the_front_waits_for_idleness_again() {
    let mut power = Power::new([true, true]);
    power.set_background(true);
    power.tick(power.last + REST_AFTER);
    assert_eq!(power.mode, Mode::Family);
    power.set_background(false);
    assert_eq!(power.mode, Mode::Normal);
    assert_eq!(power.deadline(), Some(power.last + IDLE_AFTER));
}

#[test]
fn a_quiet_window_in_front_only_slows_trame() {
    let mut power = Power::new([true, true]);
    power.tick(power.last + IDLE_AFTER);
    assert_eq!(power.mode, Mode::Own);
    assert!(!power.hidden());
    assert_eq!(power.deadline(), None);
    power.touch();
    assert_eq!(power.mode, Mode::Normal);
}

#[test]
fn the_same_window_state_again_is_not_activity() {
    let mut power = Power::new([true, true]);
    power.tick(power.last + IDLE_AFTER);
    power.set_background(false);
    assert_eq!(power.mode, Mode::Own);
}

#[test]
fn the_switch_turned_off_keeps_full_speed_everywhere() {
    let mut power = Power::new([true, true]);
    power.set_background(true);
    power.tick(power.last + REST_AFTER);
    assert_eq!(power.mode, Mode::Family);
    power.set_enabled([false, true]);
    assert_eq!(power.mode, Mode::Normal);
    assert_eq!(power.deadline(), None);
    power.tick(power.last + IDLE_AFTER);
    assert_eq!(power.mode, Mode::Normal);
}

#[test]
fn a_hidden_tab_rests_after_thirty_quiet_seconds_and_a_shown_one_never() {
    let mut power = Power::new([true, true]);
    power.set_tabs([tab(1, true), tab(2, false)]);
    let start = Instant::now();
    assert!(power.deadline().is_some_and(|at| at <= start + REST_AFTER));
    power.tick(start + REST_AFTER);
    assert_eq!(power.take_rested(), vec![SessionId(2)]);
    power.tick(start + REST_AFTER * 4);
    assert!(power.take_rested().is_empty());
}

#[test]
fn output_wakes_a_resting_tab_and_restarts_its_wait() {
    let mut power = Power::new([true, true]);
    power.set_tabs([tab(1, false)]);
    power.tick(Instant::now() + REST_AFTER);
    assert_eq!(power.take_rested(), vec![SessionId(1)]);
    power.output(SessionId(1), false);
    let woke = Instant::now();
    assert!(power.deadline().is_some_and(|at| at >= woke));
    power.tick(woke + REST_AFTER);
    assert_eq!(power.take_rested(), vec![SessionId(1)]);
}

#[test]
fn showing_a_resting_tab_wakes_it_and_hiding_it_waits_again() {
    let mut power = Power::new([true, true]);
    power.set_tabs([tab(1, false)]);
    power.tick(Instant::now() + REST_AFTER);
    power.take_rested();
    power.set_tabs([tab(1, true)]);
    assert!(!power.rest.shells()[0].1);
    power.set_tabs([tab(1, false)]);
    let hidden = Instant::now();
    power.tick(hidden + REST_AFTER / 2);
    assert!(power.take_rested().is_empty());
    power.tick(hidden + REST_AFTER);
    assert_eq!(power.take_rested(), vec![SessionId(1)]);
}

#[test]
fn a_resting_tab_in_a_shown_window_wakes_for_a_recheck_only_where_one_changes_something() {
    let mut power = Power::new([true, true]);
    power.set_tabs([tab(1, false)]);
    power.tick(Instant::now() + REST_AFTER);
    assert_eq!(power.mode, Mode::Own);
    let recheck = power.walked + FAMILY_RECHECK;
    assert_eq!(power.deadline(), RECHECKS_RESTING_SHELLS.then_some(recheck));
    power.tick(recheck);
    assert_eq!(power.walked == recheck, RECHECKS_RESTING_SHELLS);
}

#[test]
fn hidden_tabs_stay_awake_when_either_switch_is_off() {
    for switches in [[true, false], [false, true]] {
        let mut power = Power::new(switches);
        power.set_tabs([tab(1, false)]);
        power.tick(Instant::now() + REST_AFTER * 4);
        assert!(power.take_rested().is_empty());
        assert!(!power.rest.shells()[0].1);
    }
}
