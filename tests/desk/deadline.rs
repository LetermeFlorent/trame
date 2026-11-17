use std::time::Instant;

use trame::workspace::WindowSnapshot;

use crate::support::desk;

fn untouched() -> Option<WindowSnapshot> {
    panic!("a desk that owns no state never snapshots its window")
}

#[test]
fn a_fresh_desk_waits_for_nothing_and_never_snapshots() {
    let mut desk = desk();
    let now = Instant::now();
    let next = desk.next_deadline(now, untouched);
    assert!(next.effects.is_empty());
    assert_eq!(next.value, None);
    let again = desk.next_deadline(now, untouched);
    assert_eq!(again.value, next.value);
}
