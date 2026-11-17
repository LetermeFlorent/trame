use std::time::{Duration, Instant};

use trame::desk::{Desk, Effect, Grids};
use trame::session::SessionId;
use trame::theme::Appearance;

use crate::support::desk;
use crate::tabs::opened;

const PAST_REST: Duration = Duration::from_secs(120);

fn resting() -> (Desk, SessionId) {
    let (mut desk, id) = opened();
    desk.power.set_enabled([true, true]);
    (desk, id)
}

fn showing(id: SessionId) -> Grids {
    let mut grids = Grids::default();
    grids.add_pane(&[id], Some(id), (24, 80));
    grids
}

#[test]
fn a_new_appearance_repaints_once() {
    let mut desk = desk();
    let effects = desk.set_appearance(Appearance::Light);
    assert_eq!(effects.as_slice(), [Effect::Redraw]);
    assert_eq!(desk.theme.appearance, Appearance::Light);
    assert!(desk.set_appearance(Appearance::Light).is_empty());
}

#[test]
fn an_empty_desk_ticks_and_rests_quietly() {
    let mut desk = desk();
    desk.tick(Instant::now());
    desk.sync_rest(&Grids::default());
    assert!(!desk.rests());
    assert!(!desk.hidden());
}

#[test]
fn a_shown_tab_never_rests() {
    let (mut desk, id) = resting();
    desk.sync_rest(&showing(id));
    desk.power.tick(Instant::now() + PAST_REST);
    assert!(desk.power.take_rested().is_empty());
}

#[test]
fn a_hidden_tab_rests_once_quiet() {
    let (mut desk, id) = resting();
    desk.sync_rest(&Grids::default());
    desk.power.tick(Instant::now() + PAST_REST);
    assert_eq!(desk.power.take_rested(), [id]);
}

#[test]
fn a_tick_compacts_the_rested_tabs() {
    let (mut desk, _) = resting();
    desk.sync_rest(&Grids::default());
    desk.tick(Instant::now() + PAST_REST);
    assert!(desk.power.take_rested().is_empty());
}
