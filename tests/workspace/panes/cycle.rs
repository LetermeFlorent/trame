use trame::workspace::Pane;

use crate::{pane_with, tab};

fn empty_pane() -> Pane {
    Pane {
        tabs: Vec::new(),
        active: 0,
    }
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
