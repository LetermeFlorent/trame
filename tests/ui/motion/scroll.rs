use std::time::Instant;

use trame::session::SessionId;
use trame::ui::Motion;
use trame::ui::motion::DURATION;

const SESSION: SessionId = SessionId(7);

#[test]
fn a_scroll_glides_back_to_rest() {
    let start = Instant::now();
    let mut motion = Motion::new(start);
    motion.scroll_by(SESSION, 3.0, 10.0);
    assert_eq!(motion.scroll_offset(SESSION), 3.0);
    motion.tick(start + DURATION / 2);
    let middle = motion.scroll_offset(SESSION);
    assert!(middle > 0.0 && middle < 3.0, "{middle}");
    motion.tick(start + DURATION);
    assert_eq!(motion.scroll_offset(SESSION), 0.0);
    assert!(!motion.is_busy());
}

#[test]
fn quick_scrolls_add_up_within_the_limit() {
    let mut motion = Motion::new(Instant::now());
    motion.scroll_by(SESSION, 3.0, 4.0);
    motion.scroll_by(SESSION, 3.0, 4.0);
    assert_eq!(motion.scroll_offset(SESSION), 4.0);
    motion.scroll_by(SESSION, -20.0, 4.0);
    assert_eq!(motion.scroll_offset(SESSION), -4.0);
}

#[test]
fn typing_stops_the_scroll_at_once() {
    let mut motion = Motion::new(Instant::now());
    motion.scroll_by(SESSION, 3.0, 10.0);
    motion.stop_scroll(SESSION);
    assert_eq!(motion.scroll_offset(SESSION), 0.0);
    assert!(!motion.is_busy());
}
