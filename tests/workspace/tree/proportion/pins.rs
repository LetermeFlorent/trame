use trame::workspace::{Axis, Edge, PaneId};

use super::{TOLERANCE, assert_flexes, drag, flexes_of, pinned_row, pins_of};
use crate::{leaf, pinned_split, row};

#[test]
fn a_dragged_divider_pins_both_neighbours() {
    let mut root = row(vec![leaf(1), leaf(2), leaf(3)]);
    drag(&mut root, 0, 30.0);
    assert_eq!(pins_of(&root), vec![true, true, false]);
}

#[test]
fn a_later_split_keeps_pinned_sizes_and_shares_the_rest() {
    let mut root = row(vec![leaf(1), leaf(2), leaf(3)]);
    drag(&mut root, 0, 30.0);
    let before = flexes_of(&root);
    assert!(root.split(PaneId(3), PaneId(4), Edge::Right));
    let after = flexes_of(&root);
    assert_eq!(after[..2], before[..2]);
    assert!((after[2] - before[2] / 2.0).abs() < TOLERANCE);
    assert!((after[3] - before[2] / 2.0).abs() < TOLERANCE);
    assert_eq!(pins_of(&root), vec![true, true, false, false]);
}

#[test]
fn splitting_a_pinned_pane_unpins_it_and_shares_with_the_free_panes() {
    let mut root = pinned_row(vec![1.0, 2.0, 1.0], vec![true, true, false]);
    assert!(root.split(PaneId(2), PaneId(4), Edge::Right));
    assert_flexes(&root, &[1.0; 4]);
    assert_eq!(pins_of(&root), vec![true, false, false, false]);
}

#[test]
fn when_every_other_pane_is_pinned_the_target_and_new_pane_split_the_rest() {
    let mut root = pinned_row(vec![1.0, 2.0, 1.0], vec![true, false, true]);
    assert!(root.split(PaneId(2), PaneId(4), Edge::Left));
    assert_flexes(&root, &[1.0, 1.0, 1.0, 1.0]);
    assert_eq!(pins_of(&root), vec![true, false, false, true]);
}

#[test]
fn removing_a_pane_keeps_the_pins_of_the_others() {
    let mut root = pinned_row(vec![1.0, 2.0, 3.0], vec![true, false, true]);
    assert!(root.remove(PaneId(2)));
    assert_eq!(flexes_of(&root), vec![1.0, 3.0]);
    assert_eq!(pins_of(&root), vec![true, true]);
}

#[test]
fn flattening_a_nested_split_keeps_the_inner_pins() {
    let inner = pinned_split(
        Axis::Row,
        vec![leaf(2), leaf(3), leaf(4)],
        vec![1.0, 1.0, 1.0],
        vec![true, false, false],
    );
    let mut root = row(vec![leaf(1), inner]);
    assert!(root.remove(PaneId(4)));
    assert_eq!(pins_of(&root), vec![false, true, false]);
    assert_eq!(flexes_of(&root).len(), 3);
}

#[test]
fn a_pin_list_of_the_wrong_length_counts_as_unpinned() {
    let mut root = pinned_row(vec![1.0, 1.0], Vec::new());
    assert!(root.split(PaneId(2), PaneId(3), Edge::Right));
    assert_flexes(&root, &[2.0 / 3.0; 3]);
    assert_eq!(pins_of(&root), vec![false; 3]);
}

#[test]
fn a_divider_press_without_movement_pins_nothing() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    drag(&mut root, 0, 0.0);
    assert_eq!(pins_of(&root), vec![false, false]);
}
