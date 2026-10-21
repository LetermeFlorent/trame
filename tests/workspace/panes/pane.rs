use trame::workspace::Pane;

use crate::{pane_with, tab, tabs};

#[test]
fn new_pane_holds_one_active_tab() {
    let pane = Pane::new(tab(7));
    assert_eq!(pane.tabs, tabs(&[7]));
    assert_eq!(pane.active_tab(), Some(tab(7)));
}

#[test]
fn insert_without_index_appends_and_activates() {
    let mut pane = pane_with(&[1, 2], 0);
    pane.insert(tab(3), None);
    assert_eq!(pane.tabs, tabs(&[1, 2, 3]));
    assert_eq!(pane.active_tab(), Some(tab(3)));
}

#[test]
fn insert_at_index_places_the_tab_there_and_activates_it() {
    let mut pane = pane_with(&[1, 2], 1);
    pane.insert(tab(3), Some(1));
    assert_eq!(pane.tabs, tabs(&[1, 3, 2]));
    assert_eq!(pane.active_tab(), Some(tab(3)));
}

#[test]
fn insert_past_the_end_clamps_to_the_end() {
    let mut pane = pane_with(&[1, 2], 0);
    pane.insert(tab(3), Some(40));
    assert_eq!(pane.tabs, tabs(&[1, 2, 3]));
    assert_eq!(pane.active, 2);
}

#[test]
fn position_finds_known_tabs_only() {
    let pane = pane_with(&[4, 5, 6], 0);
    assert_eq!(pane.position(tab(6)), Some(2));
    assert_eq!(pane.position(tab(9)), None);
}

#[test]
fn activate_selects_a_known_tab_and_ignores_an_unknown_one() {
    let mut pane = pane_with(&[1, 2, 3], 0);
    assert!(pane.activate(tab(3)));
    assert_eq!(pane.active_tab(), Some(tab(3)));
    assert!(!pane.activate(tab(9)));
    assert_eq!(pane.active_tab(), Some(tab(3)));
}

#[test]
fn removing_an_unknown_tab_changes_nothing() {
    let mut pane = pane_with(&[1, 2], 1);
    assert!(!pane.remove(tab(9)));
    assert_eq!(pane, pane_with(&[1, 2], 1));
}

#[test]
fn removing_the_active_middle_tab_activates_the_one_that_slides_in() {
    let mut pane = pane_with(&[1, 2, 3], 1);
    assert!(pane.remove(tab(2)));
    assert_eq!(pane.tabs, tabs(&[1, 3]));
    assert_eq!(pane.active_tab(), Some(tab(3)));
}

#[test]
fn removing_the_active_last_tab_activates_the_previous_one() {
    let mut pane = pane_with(&[1, 2, 3], 2);
    pane.remove(tab(3));
    assert_eq!(pane.active_tab(), Some(tab(2)));
}

#[test]
fn removing_a_tab_before_the_active_one_keeps_that_tab_active() {
    let mut pane = pane_with(&[1, 2, 3], 2);
    pane.remove(tab(1));
    assert_eq!(pane.active_tab(), Some(tab(3)));
    let mut pane = pane_with(&[1, 2], 1);
    pane.remove(tab(1));
    assert_eq!(pane.active_tab(), Some(tab(2)));
}

#[test]
fn removing_a_tab_after_the_active_one_keeps_that_tab_active() {
    let mut pane = pane_with(&[1, 2, 3], 0);
    pane.remove(tab(3));
    assert_eq!(pane.active_tab(), Some(tab(1)));
}

#[test]
fn removing_the_only_tab_leaves_an_empty_pane() {
    let mut pane = Pane::new(tab(1));
    assert!(pane.remove(tab(1)));
    assert!(pane.tabs.is_empty());
    assert_eq!(pane.active_tab(), None);
    assert_eq!(pane.active, 0);
}
