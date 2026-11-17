use std::fs;
use std::path::Path;

use trame::host::persist::{Slot, WINDOWS_DIR, free_slot, saved_slots};

use crate::scratch::Scratch;

fn save(root: &Path, index: usize) -> Slot {
    let slot = Slot::new(Some(root), index);
    let path = slot.session_path().expect("an owned slot");
    fs::create_dir_all(path.parent().expect("a parent")).expect("slot folder");
    fs::write(path, "{}").expect("session file");
    slot
}

#[test]
fn the_saved_slots_are_the_folders_holding_a_session() {
    let scratch = Scratch::new("slot-saved");
    save(scratch.path(), 3);
    save(scratch.path(), 0);
    fs::create_dir_all(scratch.path().join(WINDOWS_DIR).join("5")).expect("empty slot");
    fs::create_dir_all(scratch.path().join(WINDOWS_DIR).join("notes")).expect("other folder");
    assert_eq!(saved_slots(scratch.path()), [0, 3]);
}

#[test]
fn a_new_window_takes_the_lowest_free_slot() {
    assert_eq!(free_slot([]), 0);
    assert_eq!(free_slot([2, 0, 1]), 3);
    assert_eq!(free_slot([0, 2]), 1);
    assert_eq!(free_slot([1]), 0);
}

#[test]
fn forgetting_a_slot_leaves_the_others() {
    let scratch = Scratch::new("slot-forget");
    let first = save(scratch.path(), 0);
    let second = save(scratch.path(), 1);
    save(scratch.path(), 2);
    second.forget().expect("forget the second");
    first.forget().expect("forget the first");
    first.forget().expect("forget twice");
    assert_eq!(saved_slots(scratch.path()), [2]);
    assert!(!Slot::new(None, 4).owns());
}
