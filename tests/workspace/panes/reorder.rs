use trame::workspace::Pane;

use crate::{pane_with, tab, tabs};

fn empty_pane() -> Pane {
    Pane {
        tabs: Vec::new(),
        active: 0,
    }
}

#[test]
fn reorder_moves_a_tab_forward_and_activates_it() {
    let mut pane = pane_with(&[1, 2, 3], 2);
    pane.reorder(tab(1), 3);
    assert_eq!(pane.tabs, tabs(&[2, 3, 1]));
    assert_eq!(pane.active_tab(), Some(tab(1)));
}

#[test]
fn reorder_moves_a_tab_backward() {
    let mut pane = pane_with(&[1, 2, 3], 0);
    pane.reorder(tab(3), 0);
    assert_eq!(pane.tabs, tabs(&[3, 1, 2]));
    assert_eq!(pane.active_tab(), Some(tab(3)));
}

#[test]
fn reorder_to_its_own_slot_or_the_slot_after_keeps_the_order() {
    let mut pane = pane_with(&[1, 2, 3], 0);
    pane.reorder(tab(2), 1);
    assert_eq!(pane.tabs, tabs(&[1, 2, 3]));
    pane.reorder(tab(2), 2);
    assert_eq!(pane.tabs, tabs(&[1, 2, 3]));
}

#[test]
fn reorder_far_past_the_end_sends_the_tab_last() {
    let mut pane = pane_with(&[1, 2, 3], 0);
    pane.reorder(tab(1), 99);
    assert_eq!(pane.tabs, tabs(&[2, 3, 1]));
}

#[test]
fn reorder_of_an_unknown_tab_is_a_no_op() {
    let mut pane = pane_with(&[1, 2, 3], 1);
    pane.reorder(tab(9), 0);
    assert_eq!(pane, pane_with(&[1, 2, 3], 1));
}

#[test]
fn cycle_forward_wraps_to_the_first_tab() {
    let mut pane = pane_with(&[1, 2, 3], 2);
    pane.cycle(1);
    assert_eq!(pane.active_tab(), Some(tab(1)));
}

#[test]
fn cycle_backward_wraps_to_the_last_tab() {
    let mut pane = pane_with(&[1, 2, 3], 0);
    pane.cycle(-1);
    assert_eq!(pane.active_tab(), Some(tab(3)));
}

#[test]
fn cycle_by_more_than_the_tab_count_still_wraps() {
    let mut pane = pane_with(&[1, 2, 3], 1);
    pane.cycle(7);
    assert_eq!(pane.active, 2);
    pane.cycle(-8);
    assert_eq!(pane.active, 0);
}

#[test]
fn cycle_on_an_empty_pane_does_nothing() {
    let mut pane = empty_pane();
    pane.cycle(1);
    pane.cycle(-1);
    assert_eq!(pane, empty_pane());
}
