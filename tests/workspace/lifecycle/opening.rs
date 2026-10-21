use trame::workspace::{Edge, InstanceId, TabLocation, Workspace};

use crate::{pane_of, tab, tabs, workspace_with};

#[test]
fn a_new_workspace_is_empty_with_the_sidebar_open() {
    let workspace = Workspace::new();
    assert!(workspace.is_empty());
    assert!(workspace.sidebar_open);
    assert!(workspace.active_instance().is_none());
    assert_eq!(workspace.active_tab(), None);
    assert!(workspace.tabs().is_empty());
    assert_eq!(workspace, Workspace::default());
}

#[test]
fn add_instance_numbers_instances_and_activates_the_newest() {
    let mut workspace = Workspace::new();
    assert_eq!(workspace.add_instance(None, tab(1)), InstanceId(1));
    assert_eq!(workspace.add_instance(None, tab(2)), InstanceId(2));
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.active_tab(), Some(tab(2)));
    assert!(!workspace.is_empty());
}

#[test]
fn add_instance_keeps_a_given_name_and_numbers_the_others() {
    let mut workspace = Workspace::new();
    workspace.add_instance(Some("build".to_string()), tab(1));
    workspace.add_instance(None, tab(2));
    assert_eq!(workspace.instances[0].name, "build");
    assert_eq!(workspace.instances[1].name, "Instance 2");
}

#[test]
fn add_tab_on_an_empty_workspace_opens_an_instance() {
    let mut workspace = Workspace::new();
    workspace.add_tab(tab(1));
    assert_eq!(workspace.instances.len(), 1);
    assert_eq!(workspace.active_tab(), Some(tab(1)));
}

#[test]
fn add_tab_goes_right_after_the_active_tab_and_becomes_active() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.activate_tab(tab(1));
    workspace.add_tab(tab(3));
    assert_eq!(workspace.tabs(), tabs(&[1, 3, 2]));
    assert_eq!(workspace.active_tab(), Some(tab(3)));
}

#[test]
fn split_active_on_an_empty_workspace_opens_an_instance() {
    let mut workspace = Workspace::new();
    workspace.split_active(tab(1), Edge::Right);
    assert_eq!(workspace.instances.len(), 1);
    assert_eq!(workspace.active_tab(), Some(tab(1)));
}

#[test]
fn split_active_adds_a_pane_after_the_active_one_and_focuses_it() {
    let mut workspace = workspace_with(&[1]);
    workspace.split_active(tab(2), Edge::Right);
    let panes = workspace.instances[0].root.panes();
    assert_eq!(panes, vec![pane_of(&workspace, 1), pane_of(&workspace, 2)]);
    assert_eq!(workspace.instances[0].active_pane, pane_of(&workspace, 2));
    assert_eq!(workspace.active_tab(), Some(tab(2)));
}

#[test]
fn split_active_on_the_left_edge_puts_the_new_pane_first() {
    let mut workspace = workspace_with(&[1]);
    workspace.split_active(tab(2), Edge::Left);
    assert_eq!(workspace.tabs(), tabs(&[2, 1]));
}

#[test]
fn locate_reports_instance_pane_and_position() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.add_instance(None, tab(3));
    workspace.split_active(tab(4), Edge::Bottom);
    let expected = TabLocation {
        instance: 0,
        pane: pane_of(&workspace, 2),
        index: 1,
    };
    assert_eq!(workspace.locate(tab(2)), Some(expected));
    assert_eq!(
        workspace.locate(tab(4)).map(|found| found.instance),
        Some(1)
    );
    assert_eq!(workspace.locate(tab(4)).map(|found| found.index), Some(0));
}

#[test]
fn locate_of_an_unknown_tab_is_none() {
    assert_eq!(workspace_with(&[1]).locate(tab(9)), None);
}

#[test]
fn activate_tab_switches_instance_pane_and_tab() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.split_active(tab(3), Edge::Right);
    workspace.add_instance(None, tab(4));
    workspace.activate_tab(tab(1));
    assert_eq!(workspace.active, 0);
    assert_eq!(workspace.instances[0].active_pane, pane_of(&workspace, 1));
    assert_eq!(workspace.active_tab(), Some(tab(1)));
}

#[test]
fn activate_tab_selects_a_background_tab_of_its_pane() {
    let mut workspace = workspace_with(&[1, 2, 3]);
    workspace.activate_tab(tab(2));
    assert_eq!(workspace.active_tab(), Some(tab(2)));
}

#[test]
fn activate_tab_of_an_unknown_tab_changes_nothing() {
    let mut workspace = workspace_with(&[1, 2]);
    let before = workspace.clone();
    workspace.activate_tab(tab(9));
    assert_eq!(workspace, before);
}

#[test]
fn tabs_lists_every_tab_instance_by_instance() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.add_instance(None, tab(3));
    workspace.add_tab(tab(4));
    assert_eq!(workspace.tabs(), tabs(&[1, 2, 3, 4]));
}
