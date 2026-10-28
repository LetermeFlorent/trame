use std::time::{Duration, Instant};

use super::{RECHECK, Visibility};

#[test]
fn the_visibility_retry_backs_off() {
    let mut visibility = Visibility::default();
    let start = Instant::now();
    visibility.missed_at(start);
    let mut due = visibility.retry.expect("a missed frame is retried");
    let mut waits = vec![due - start];
    for _ in 0..5 {
        let next = visibility.still_hidden(due);
        waits.push(next - due);
        due = next;
    }
    let expected = [250, 500, 1000, 2000, 2000, 2000].map(Duration::from_millis);
    assert_eq!(waits, expected);
}

#[test]
fn a_drawn_frame_starts_the_wait_over() {
    let mut visibility = Visibility::default();
    let start = Instant::now();
    visibility.missed_at(start);
    visibility.still_hidden(start);
    visibility.still_hidden(start);
    visibility.drawn();
    visibility.missed_at(start);
    assert_eq!(visibility.retry, Some(start + RECHECK));
}

#[test]
fn another_missed_frame_keeps_the_retry_already_planned() {
    let mut visibility = Visibility::default();
    let start = Instant::now();
    visibility.missed_at(start);
    visibility.missed_at(start + RECHECK / 2);
    assert_eq!(visibility.retry, Some(start + RECHECK));
}
