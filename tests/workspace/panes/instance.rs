use trame::workspace::{Edge, Instance, InstanceId, PaneId};

use crate::{leaf, row, tab, tabs};

fn single() -> Instance {
    Instance::new(InstanceId(1), "shell".to_string(), PaneId(1), tab(1))
}

fn three_in_a_row() -> Instance {
    let mut instance = single();
    instance.split(PaneId(1), PaneId(2), tab(2), Edge::Right);
    instance.split(PaneId(2), PaneId(3), tab(3), Edge::Right);
    instance
}

#[test]
fn a_new_instance_has_one_pane_holding_one_tab() {
    let instance = single();
    assert_eq!(instance.id, InstanceId(1));
    assert_eq!(instance.name, "shell");
    assert_eq!(instance.root, leaf(1));
    assert_eq!(instance.active_pane, PaneId(1));
    assert_eq!(instance.active_tab(), Some(tab(1)));
    assert_eq!(instance.tab_count(), 1);
    assert!(!instance.is_empty());
}

#[test]
fn split_adds_a_pane_next_to_the_target_and_focuses_it() {
    let mut instance = single();
    assert!(instance.split(PaneId(1), PaneId(2), tab(2), Edge::Right));
    assert_eq!(instance.root, row(vec![leaf(1), leaf(2)]));
    assert_eq!(instance.active_pane, PaneId(2));
    assert_eq!(instance.active_tab(), Some(tab(2)));
}

#[test]
fn split_of_a_missing_pane_is_refused_and_adds_nothing() {
    let mut instance = single();
    assert!(!instance.split(PaneId(9), PaneId(2), tab(2), Edge::Right));
    assert_eq!(instance, single());
}

#[test]
fn tabs_follow_the_layout_order_not_the_pane_ids() {
    let mut instance = single();
    instance.split(PaneId(1), PaneId(2), tab(2), Edge::Left);
    assert_eq!(instance.tabs(), tabs(&[2, 1]));
}

#[test]
fn find_returns_the_pane_holding_a_tab() {
    let instance = three_in_a_row();
    assert_eq!(instance.find(tab(2)), Some(PaneId(2)));
    assert_eq!(instance.find(tab(9)), None);
}

#[test]
fn focus_ignores_panes_that_do_not_exist() {
    let mut instance = three_in_a_row();
    instance.focus(PaneId(1));
    assert_eq!(instance.active_pane, PaneId(1));
    instance.focus(PaneId(9));
    assert_eq!(instance.active_pane, PaneId(1));
}

#[test]
fn removing_an_unknown_tab_returns_false() {
    let mut instance = three_in_a_row();
    assert!(!instance.remove_tab(tab(9)));
    assert_eq!(instance, three_in_a_row());
}

#[test]
fn removing_the_last_tab_of_a_background_pane_keeps_the_focus() {
    let mut instance = three_in_a_row();
    assert!(instance.remove_tab(tab(1)));
    assert_eq!(instance.active_pane, PaneId(3));
    assert_eq!(instance.root.panes(), vec![PaneId(2), PaneId(3)]);
}

#[test]
fn removing_the_last_tab_of_the_focused_pane_focuses_the_previous_pane() {
    let mut instance = three_in_a_row();
    instance.focus(PaneId(2));
    instance.remove_tab(tab(2));
    assert_eq!(instance.active_pane, PaneId(1));
    assert_eq!(instance.tabs(), tabs(&[1, 3]));
}

#[test]
fn removing_the_last_tab_of_the_first_focused_pane_focuses_the_next_one() {
    let mut instance = three_in_a_row();
    instance.focus(PaneId(1));
    instance.remove_tab(tab(1));
    assert_eq!(instance.active_pane, PaneId(2));
}

#[test]
fn removing_the_only_tab_empties_the_instance() {
    let mut instance = single();
    assert!(instance.remove_tab(tab(1)));
    assert!(instance.is_empty());
    assert_eq!(instance.tab_count(), 0);
    assert_eq!(instance.active_tab(), None);
    assert!(instance.tabs().is_empty());
}
