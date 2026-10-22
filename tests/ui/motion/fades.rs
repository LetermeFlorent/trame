use std::time::Instant;

use trame::session::SessionId;
use trame::ui::motion::DURATION;
use trame::ui::{Hit, Motion};
use trame::workspace::PaneId;

const TAB: Hit = Hit::Tab(PaneId(0), SessionId(1));
const CLOSE: Hit = Hit::TabClose(PaneId(0), SessionId(1));

#[test]
fn hovering_fades_in_then_out() {
    let start = Instant::now();
    let mut motion = Motion::new(start);
    motion.hover(Some(TAB));
    assert_eq!(motion.glow(TAB), 0.0);
    assert!(motion.is_busy());
    motion.tick(start + DURATION / 2);
    let middle = motion.glow(TAB);
    assert!(middle > 0.0 && middle < 1.0, "{middle}");
    motion.tick(start + DURATION);
    assert_eq!(motion.glow(TAB), 1.0);
    assert!(!motion.is_busy());
    motion.hover(None);
    motion.tick(start + DURATION * 2);
    assert_eq!(motion.glow(TAB), 0.0);
}

#[test]
fn hovering_a_close_button_keeps_its_tab_lit() {
    let start = Instant::now();
    let mut motion = Motion::new(start);
    motion.hover(Some(CLOSE));
    motion.tick(start + DURATION);
    assert_eq!(motion.glow(CLOSE), 1.0);
    assert_eq!(motion.glow(TAB), 1.0);
}

#[test]
fn moving_from_the_close_button_to_its_tab_keeps_the_tab_lit() {
    let start = Instant::now();
    let mut motion = Motion::new(start);
    motion.hover(Some(CLOSE));
    motion.tick(start + DURATION);
    motion.hover(Some(TAB));
    motion.tick(start + DURATION * 2);
    assert_eq!(motion.glow(TAB), 1.0);
    assert_eq!(motion.glow(CLOSE), 0.0);
}

#[test]
fn an_unhovered_surface_has_no_glow() {
    let motion = Motion::new(Instant::now());
    assert_eq!(motion.glow(TAB), 0.0);
    assert!(!motion.is_busy());
}
