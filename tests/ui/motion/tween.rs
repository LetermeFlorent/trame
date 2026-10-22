use std::time::Instant;

use trame::ui::motion::{DURATION, Tween, ease_out};

#[test]
fn easing_starts_fast_and_lands_on_the_end() {
    assert_eq!(ease_out(0.0), 0.0);
    assert_eq!(ease_out(1.0), 1.0);
    assert!(ease_out(0.5) > 0.5);
    assert_eq!(ease_out(-1.0), 0.0);
    assert_eq!(ease_out(2.0), 1.0);
}

#[test]
fn a_tween_moves_between_its_ends_then_stops() {
    let start = Instant::now();
    let tween = Tween::between(0.0_f32, 10.0, start);
    assert_eq!(tween.value(start), 0.0);
    let middle = tween.value(start + DURATION / 2);
    assert!(middle > 5.0 && middle < 10.0, "{middle}");
    assert!(!tween.is_done(start + DURATION / 2));
    assert!(tween.is_done(start + DURATION));
    assert_eq!(tween.value(start + DURATION * 3), 10.0);
}

#[test]
fn retargeting_starts_from_the_current_value() {
    let start = Instant::now();
    let mut tween = Tween::between(0.0_f32, 10.0, start);
    let halfway = start + DURATION / 2;
    let reached = tween.value(halfway);
    tween.retarget(0.0, halfway);
    assert_eq!(tween.value(halfway), reached);
    assert_eq!(tween.target(), 0.0);
    assert_eq!(tween.value(halfway + DURATION), 0.0);
}

#[test]
fn a_snapped_tween_is_already_done() {
    let start = Instant::now();
    let mut tween = Tween::between(0.0_f32, 10.0, start);
    tween.snap(4.0);
    assert!(tween.is_done(start));
    assert_eq!(tween.value(start), 4.0);
}

#[test]
fn a_settled_tween_never_moves() {
    let start = Instant::now();
    let tween = Tween::settled(3.0_f32, start);
    assert!(tween.is_done(start));
    assert_eq!(tween.value(start + DURATION / 2), 3.0);
}
