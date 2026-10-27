use trame::workspace::{DropTarget, Edge, InstanceId, PaneId, Workspace};

use crate::{pane_of, pane_tabs, tab, tabs, workspace_with};

fn swap_with(workspace: &Workspace, instance: usize, pane: PaneId) -> DropTarget {
    DropTarget::Swap {
        instance: workspace.instances[instance].id,
        pane,
    }
}

fn two_panes(left: &[u64], right: &[u64]) -> Workspace {
    let mut workspace = workspace_with(left);
    workspace.split_active(tab(right[0]), Edge::Right);
    for id in &right[1..] {
        workspace.add_tab(tab(*id));
    }
    workspace
}

#[test]
fn swapping_two_single_tab_panes_exchanges_their_places() {
    let mut workspace = two_panes(&[1], &[2]);
    let (left, right) = (pane_of(&workspace, 1), pane_of(&workspace, 2));
    let root = workspace.instances[0].root.clone();
    assert!(workspace.move_tab(tab(1), swap_with(&workspace, 0, right)));
    assert_eq!(pane_tabs(&workspace, left), tabs(&[2]));
    assert_eq!(pane_tabs(&workspace, right), tabs(&[1]));
    assert_eq!(workspace.instances[0].root, root);
    assert_eq!(workspace.instances[0].active_pane, right);
    assert_eq!(workspace.active_tab(), Some(tab(1)));
}

#[test]
fn swapping_only_exchanges_the_dragged_tab_with_the_visible_one() {
    let mut workspace = two_panes(&[1, 2, 3], &[4, 5]);
    let (left, right) = (pane_of(&workspace, 1), pane_of(&workspace, 4));
    assert!(workspace.move_tab(tab(2), swap_with(&workspace, 0, right)));
    assert_eq!(pane_tabs(&workspace, left), tabs(&[1, 5, 3]));
    assert_eq!(pane_tabs(&workspace, right), tabs(&[4, 2]));
    assert_eq!(workspace.instances[0].panes[&left].active, 1);
    assert_eq!(workspace.active_tab(), Some(tab(2)));
}

#[test]
fn swapping_onto_the_own_pane_is_refused() {
    let mut workspace = two_panes(&[1, 2], &[3]);
    let before = workspace.clone();
    let pane = pane_of(&workspace, 1);
    assert!(!workspace.move_tab(tab(2), swap_with(&workspace, 0, pane)));
    assert_eq!(workspace, before);
}

#[test]
fn swapping_with_a_missing_pane_or_instance_is_refused() {
    let mut workspace = two_panes(&[1], &[2]);
    let before = workspace.clone();
    assert!(!workspace.move_tab(tab(1), swap_with(&workspace, 0, PaneId(99))));
    let lost = DropTarget::Swap {
        instance: InstanceId(99),
        pane: pane_of(&workspace, 2),
    };
    assert!(!workspace.move_tab(tab(1), lost));
    assert_eq!(workspace, before);
}

#[test]
fn swapping_across_instances_keeps_both_instances() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.add_instance(None, tab(3));
    let target = swap_with(&workspace, 1, pane_of(&workspace, 3));
    assert!(workspace.move_tab(tab(1), target));
    assert_eq!(workspace.instances[0].tabs(), tabs(&[3, 2]));
    assert_eq!(workspace.instances[1].tabs(), tabs(&[1]));
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.active_tab(), Some(tab(1)));
}
