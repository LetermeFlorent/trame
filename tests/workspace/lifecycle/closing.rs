use trame::workspace::{Edge, InstanceId, Workspace};

use crate::{pane_of, tab, tabs, workspace_with};

fn three_panes() -> Workspace {
    let mut workspace = workspace_with(&[1]);
    workspace.split_active(tab(2), Edge::Right);
    workspace.split_active(tab(3), Edge::Right);
    workspace
}

fn three_instances() -> Workspace {
    let mut workspace = workspace_with(&[1]);
    workspace.add_instance(None, tab(2));
    workspace.add_instance(None, tab(3));
    workspace
}

#[test]
fn closing_an_unknown_tab_returns_false_and_changes_nothing() {
    let mut workspace = workspace_with(&[1, 2]);
    let before = workspace.clone();
    assert!(!workspace.close_tab(tab(9)));
    assert_eq!(workspace, before);
}

#[test]
fn closing_a_tab_keeps_its_pane_while_other_tabs_remain() {
    let mut workspace = workspace_with(&[1, 2, 3]);
    assert!(workspace.close_tab(tab(2)));
    assert_eq!(workspace.tabs(), tabs(&[1, 3]));
    assert_eq!(workspace.instances[0].panes.len(), 1);
}

#[test]
fn closing_the_last_tab_of_a_pane_removes_the_pane() {
    let mut workspace = three_panes();
    let middle = pane_of(&workspace, 2);
    assert!(workspace.close_tab(tab(2)));
    let instance = &workspace.instances[0];
    assert!(!instance.root.contains(middle));
    assert!(!instance.panes.contains_key(&middle));
    assert_eq!(workspace.tabs(), tabs(&[1, 3]));
}

#[test]
fn closing_the_focused_pane_focuses_the_previous_one() {
    let mut workspace = three_panes();
    workspace.activate_tab(tab(2));
    workspace.close_tab(tab(2));
    assert_eq!(workspace.instances[0].active_pane, pane_of(&workspace, 1));
}

#[test]
fn closing_a_background_pane_keeps_the_focus() {
    let mut workspace = three_panes();
    workspace.close_tab(tab(1));
    assert_eq!(workspace.instances[0].active_pane, pane_of(&workspace, 3));
}

#[test]
fn closing_the_last_tab_of_an_instance_removes_the_instance() {
    let mut workspace = three_instances();
    let first = workspace.instances[0].id;
    assert!(workspace.close_tab(tab(1)));
    assert_eq!(workspace.instances.len(), 2);
    assert_eq!(workspace.instance_index(first), None);
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.active_tab(), Some(tab(3)));
}

#[test]
fn closing_every_tab_leaves_the_workspace_empty() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.close_tab(tab(1));
    assert!(!workspace.is_empty());
    workspace.close_tab(tab(2));
    assert!(workspace.is_empty());
    assert_eq!(workspace.active_tab(), None);
    assert_eq!(workspace.active, 0);
}

#[test]
fn remove_instance_hands_back_its_tabs_in_layout_order() {
    let mut workspace = workspace_with(&[1]);
    workspace.split_active(tab(2), Edge::Left);
    assert_eq!(workspace.remove_instance(0), tabs(&[2, 1]));
    assert!(workspace.is_empty());
}

#[test]
fn remove_instance_out_of_range_returns_nothing() {
    let mut workspace = three_instances();
    assert!(workspace.remove_instance(3).is_empty());
    assert_eq!(workspace.instances.len(), 3);
}

#[test]
fn removing_an_earlier_instance_keeps_the_same_instance_active() {
    let mut workspace = three_instances();
    workspace.remove_instance(0);
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.active_tab(), Some(tab(3)));
}

#[test]
fn removing_a_later_instance_keeps_the_same_instance_active() {
    let mut workspace = three_instances();
    workspace.activate_tab(tab(1));
    workspace.remove_instance(2);
    assert_eq!(workspace.active, 0);
    assert_eq!(workspace.active_tab(), Some(tab(1)));
}

#[test]
fn removing_the_active_last_instance_activates_the_new_last_one() {
    let mut workspace = three_instances();
    workspace.remove_instance(2);
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.active_tab(), Some(tab(2)));
}

#[test]
fn removing_the_active_middle_instance_activates_the_one_that_follows() {
    let mut workspace = three_instances();
    workspace.activate_tab(tab(2));
    workspace.remove_instance(1);
    assert_eq!(workspace.active_tab(), Some(tab(3)));
}

#[test]
fn instance_index_follows_ids_as_instances_shift() {
    let mut workspace = three_instances();
    assert_eq!(workspace.instance_index(InstanceId(3)), Some(2));
    workspace.remove_instance(0);
    assert_eq!(workspace.instance_index(InstanceId(3)), Some(1));
    assert_eq!(workspace.instance_index(InstanceId(1)), None);
}
