use std::time::Instant;

use trame::desk::{Effect, Grids};
use trame::theme::Appearance;

use crate::support::desk;

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
