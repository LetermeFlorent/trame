use trame::workspace::{DropTarget, Edge, PaneId, Workspace};

use crate::{assert_unchanged, column, leaf, pane_of, pane_tabs, row, tab, tabs, workspace_with};

fn split_onto(workspace: &Workspace, pane: PaneId, edge: Edge) -> DropTarget {
    DropTarget::Split {
        instance: workspace.instances[0].id,
        pane,
        edge,
    }
}

fn two_panes() -> Workspace {
    let mut workspace = workspace_with(&[1, 2, 3]);
    workspace.split_active(tab(4), Edge::Right);
    workspace
}

fn two_instances() -> Workspace {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.add_instance(None, tab(3));
    workspace
}

#[test]
fn dropping_on_a_pane_edge_opens_a_new_pane_holding_the_tab() {
    let mut workspace = two_panes();
    let (left, right) = (pane_of(&workspace, 1), pane_of(&workspace, 4));
    assert!(workspace.move_tab(tab(3), split_onto(&workspace, right, Edge::Bottom)));
    let created = pane_of(&workspace, 3);
    let nested = column(vec![leaf(right.0), leaf(created.0)]);
    assert_eq!(workspace.instances[0].root, row(vec![leaf(left.0), nested]));
    assert_eq!(pane_tabs(&workspace, created), tabs(&[3]));
    assert_eq!(pane_tabs(&workspace, left), tabs(&[1, 2]));
    assert_eq!(workspace.instances[0].active_pane, created);
}

#[test]
fn dropping_on_the_leading_edge_puts_the_new_pane_first() {
    let mut workspace = two_panes();
    let right = pane_of(&workspace, 4);
    assert!(workspace.move_tab(tab(1), split_onto(&workspace, right, Edge::Top)));
    let (left, created) = (pane_of(&workspace, 2), pane_of(&workspace, 1));
    let nested = column(vec![leaf(created.0), leaf(right.0)]);
    assert_eq!(workspace.instances[0].root, row(vec![leaf(left.0), nested]));
}

#[test]
fn a_pane_cannot_be_split_with_its_own_only_tab() {
    let mut workspace = two_panes();
    let before = workspace.clone();
    let alone = pane_of(&workspace, 4);
    assert!(!workspace.move_tab(tab(4), split_onto(&workspace, alone, Edge::Left)));
    assert_eq!(workspace, before);
}

#[test]
fn a_pane_can_be_split_with_one_of_its_several_tabs() {
    let mut workspace = workspace_with(&[1, 2, 3]);
    let source = pane_of(&workspace, 1);
    assert!(workspace.move_tab(tab(2), split_onto(&workspace, source, Edge::Right)));
    let created = pane_of(&workspace, 2);
    assert_eq!(pane_tabs(&workspace, source), tabs(&[1, 3]));
    assert_eq!(pane_tabs(&workspace, created), tabs(&[2]));
    assert_eq!(workspace.instances[0].root.panes(), vec![source, created]);
}

#[test]
fn dropping_on_a_missing_pane_is_refused_and_keeps_every_tab() {
    let mut workspace = two_panes();
    let before = workspace.clone();
    assert!(!workspace.move_tab(tab(2), split_onto(&workspace, PaneId(99), Edge::Left)));
    assert_unchanged(&workspace, &before);
}

#[test]
fn splitting_with_the_only_tab_of_another_pane_collapses_the_source_pane() {
    let mut workspace = workspace_with(&[1]);
    workspace.split_active(tab(2), Edge::Right);
    workspace.split_active(tab(3), Edge::Right);
    let target = pane_of(&workspace, 3);
    assert!(workspace.move_tab(tab(1), split_onto(&workspace, target, Edge::Bottom)));
    let instance = &workspace.instances[0];
    assert_eq!(instance.panes.len(), 3);
    assert_eq!(instance.tab_count(), 3);
    assert_eq!(instance.root.panes().len(), 3);
    assert_eq!(workspace.tabs(), tabs(&[2, 3, 1]));
}

#[test]
fn dropping_on_an_instance_appends_to_its_active_pane() {
    let mut workspace = two_instances();
    let target = DropTarget::Instance(workspace.instances[1].id);
    assert!(workspace.move_tab(tab(1), target));
    assert_eq!(workspace.instances[0].tabs(), tabs(&[2]));
    assert_eq!(workspace.instances[1].tabs(), tabs(&[3, 1]));
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.active_tab(), Some(tab(1)));
}

#[test]
fn dropping_on_the_own_instance_is_refused() {
    let mut workspace = two_instances();
    let before = workspace.clone();
    let target = DropTarget::Instance(workspace.instances[0].id);
    assert!(!workspace.move_tab(tab(1), target));
    assert_eq!(workspace, before);
}

#[test]
fn dropping_the_last_tab_on_another_instance_removes_the_source_instance() {
    let mut workspace = two_instances();
    let target = DropTarget::Instance(workspace.instances[0].id);
    assert!(workspace.move_tab(tab(3), target));
    assert_eq!(workspace.instances.len(), 1);
    assert_eq!(workspace.active, 0);
    assert_eq!(workspace.tabs(), tabs(&[1, 2, 3]));
}

#[test]
fn dropping_on_a_new_instance_detaches_the_tab() {
    let mut workspace = two_instances();
    assert!(workspace.move_tab(tab(2), DropTarget::NewInstance));
    assert_eq!(workspace.instances.len(), 3);
    assert_eq!(workspace.instances[0].tabs(), tabs(&[1]));
    assert_eq!(workspace.instances[2].tabs(), tabs(&[2]));
    assert_eq!(workspace.instances[2].name, "Instance 3");
    assert_eq!(workspace.active, 2);
}

#[test]
fn the_only_tab_of_an_instance_cannot_leave_for_a_new_instance() {
    let mut workspace = two_instances();
    let before = workspace.clone();
    assert!(!workspace.move_tab(tab(3), DropTarget::NewInstance));
    assert_eq!(workspace, before);
}

#[test]
fn a_pane_alone_in_its_cell_moves_to_a_new_instance_and_collapses_the_tree() {
    let mut workspace = workspace_with(&[1]);
    workspace.split_active(tab(2), Edge::Right);
    let kept = pane_of(&workspace, 1);
    assert!(workspace.move_tab(tab(2), DropTarget::NewInstance));
    assert_eq!(workspace.instances[0].root, leaf(kept.0));
    assert_eq!(workspace.instances[1].tabs(), tabs(&[2]));
}
