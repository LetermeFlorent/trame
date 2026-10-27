use std::time::{Duration, Instant};

use trame::ui::Motion;

const TALL: f32 = 500.0;
const SHORT: f32 = 200.0;

#[test]
fn the_settings_window_opens_at_its_height_then_glides_to_a_new_one() {
    let start = Instant::now();
    let mut motion = Motion::new(start);
    assert_eq!(motion.popup_height(Some(TALL)), Some(TALL));
    assert!(!motion.is_busy());
    assert_eq!(motion.popup_height(Some(SHORT)), Some(TALL));
    assert!(motion.is_busy());
    motion.tick(start + Duration::from_millis(60));
    let middle = motion.popup_height(Some(SHORT)).expect("a height");
    assert!(middle < TALL && middle > SHORT, "{middle}");
    motion.tick(start + Duration::from_millis(200));
    assert_eq!(motion.popup_height(Some(SHORT)), Some(SHORT));
    assert!(!motion.is_busy());
}

#[test]
fn a_closed_window_opens_again_without_gliding() {
    let start = Instant::now();
    let mut motion = Motion::new(start);
    motion.popup_height(Some(TALL));
    assert_eq!(motion.popup_height(None), None);
    assert_eq!(motion.popup_height(Some(SHORT)), Some(SHORT));
    assert!(!motion.is_busy());
}
