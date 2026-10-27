use trame::workspace::{DropTarget, InstanceId, PaneId, Workspace};

use super::{two_instances, two_panes};
use crate::{leaf, pane_of, pane_tabs, tab, tabs, workspace_with};

fn into_pane(workspace: &Workspace, pane: PaneId, index: Option<usize>) -> DropTarget {
    DropTarget::Tabs {
        instance: workspace.instances[0].id,
        pane,
        index,
    }
}

#[test]
fn moving_an_unknown_tab_is_refused_for_every_target() {
    let mut workspace = two_panes();
    let before = workspace.clone();
    let pane = pane_of(&workspace, 1);
    let instance = workspace.instances[0].id;
    assert!(!workspace.move_tab(tab(9), into_pane(&before, pane, None)));
    assert!(!workspace.move_tab(tab(9), DropTarget::Instance(instance)));
    assert!(!workspace.move_tab(tab(9), DropTarget::NewInstance));
    assert_eq!(workspace, before);
}

#[test]
fn dropping_in_the_own_pane_reorders_and_activates_the_tab() {
    let mut workspace = workspace_with(&[1, 2, 3]);
    let pane = pane_of(&workspace, 1);
    assert!(workspace.move_tab(tab(3), into_pane(&workspace, pane, Some(0))));
    assert_eq!(pane_tabs(&workspace, pane), tabs(&[3, 1, 2]));
    assert_eq!(workspace.active_tab(), Some(tab(3)));
}

#[test]
fn dropping_in_the_own_pane_without_index_sends_the_tab_last() {
    let mut workspace = workspace_with(&[1, 2, 3]);
    let pane = pane_of(&workspace, 1);
    assert!(workspace.move_tab(tab(1), into_pane(&workspace, pane, None)));
    assert_eq!(pane_tabs(&workspace, pane), tabs(&[2, 3, 1]));
}

#[test]
fn dropping_a_tab_on_its_own_slot_leaves_the_order_alone() {
    let mut workspace = workspace_with(&[1, 2, 3]);
    let pane = pane_of(&workspace, 1);
    workspace.move_tab(tab(2), into_pane(&workspace, pane, Some(1)));
    workspace.move_tab(tab(2), into_pane(&workspace, pane, Some(2)));
    assert_eq!(pane_tabs(&workspace, pane), tabs(&[1, 2, 3]));
    assert_eq!(workspace.active_tab(), Some(tab(2)));
}

#[test]
fn dropping_in_another_pane_inserts_at_the_index_and_focuses_that_pane() {
    let mut workspace = two_panes();
    let (source, target) = (pane_of(&workspace, 1), pane_of(&workspace, 4));
    assert!(workspace.move_tab(tab(2), into_pane(&workspace, target, Some(0))));
    assert_eq!(pane_tabs(&workspace, target), tabs(&[2, 4]));
    assert_eq!(pane_tabs(&workspace, source), tabs(&[1, 3]));
    assert_eq!(workspace.instances[0].active_pane, target);
    assert_eq!(workspace.active_tab(), Some(tab(2)));
}

#[test]
fn dropping_in_another_pane_without_index_appends() {
    let mut workspace = two_panes();
    let target = pane_of(&workspace, 4);
    assert!(workspace.move_tab(tab(1), into_pane(&workspace, target, None)));
    assert_eq!(pane_tabs(&workspace, target), tabs(&[4, 1]));
}

#[test]
fn moving_the_only_tab_of_a_pane_out_removes_that_pane() {
    let mut workspace = two_panes();
    let (source, emptied) = (pane_of(&workspace, 1), pane_of(&workspace, 4));
    assert!(workspace.move_tab(tab(4), into_pane(&workspace, source, None)));
    let instance = &workspace.instances[0];
    assert_eq!(instance.root, leaf(source.0));
    assert!(!instance.panes.contains_key(&emptied));
    assert_eq!(instance.active_pane, source);
    assert_eq!(pane_tabs(&workspace, source), tabs(&[1, 2, 3, 4]));
}

#[test]
fn dropping_in_a_missing_pane_is_refused_and_restores_the_workspace() {
    let mut workspace = two_panes();
    let before = workspace.clone();
    assert!(!workspace.move_tab(tab(2), into_pane(&workspace, PaneId(99), None)));
    assert_eq!(workspace, before);
}

#[test]
fn dropping_in_a_missing_instance_is_refused() {
    let mut workspace = two_panes();
    let before = workspace.clone();
    let target = DropTarget::Tabs {
        instance: InstanceId(99),
        pane: pane_of(&workspace, 1),
        index: None,
    };
    assert!(!workspace.move_tab(tab(2), target));
    assert_eq!(workspace, before);
}

#[test]
fn dropping_in_another_instance_moves_the_tab_and_activates_that_instance() {
    let mut workspace = two_instances();
    let target = DropTarget::Tabs {
        instance: workspace.instances[1].id,
        pane: pane_of(&workspace, 3),
        index: Some(0),
    };
    assert!(workspace.move_tab(tab(1), target));
    assert_eq!(workspace.instances[0].tabs(), tabs(&[2]));
    assert_eq!(workspace.instances[1].tabs(), tabs(&[1, 3]));
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.active_tab(), Some(tab(1)));
}

#[test]
fn moving_the_last_tab_of_an_instance_removes_that_instance() {
    let mut workspace = workspace_with(&[1]);
    workspace.add_instance(None, tab(2));
    let target = DropTarget::Tabs {
        instance: workspace.instances[1].id,
        pane: pane_of(&workspace, 2),
        index: None,
    };
    assert!(workspace.move_tab(tab(1), target));
    assert_eq!(workspace.instances.len(), 1);
    assert_eq!(workspace.active, 0);
    assert_eq!(workspace.tabs(), tabs(&[2, 1]));
}
