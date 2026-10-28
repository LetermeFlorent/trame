use trame::session::SessionId;
use trame::workspace::{Axis, NodeSnapshot, PaneSnapshot, SnapshotError, TabSnapshot, Workspace};

use super::{instance_of, pane, restore, shape, single};

fn row(ratios: Vec<f32>, children: Vec<NodeSnapshot>) -> NodeSnapshot {
    NodeSnapshot::Split {
        axis: Axis::Row,
        ratios,
        pinned: Vec::new(),
        children,
    }
}

#[test]
fn unusable_ratios_fall_back_to_equal_shares() {
    let root = row(vec![0.0, -1.0, f32::NAN], vec![pane(1), pane(1), pane(1)]);
    let (restored, _) = restore(&single(root));
    assert_eq!(shape(&restored.instances[0]), "Row(1:[1],1:[1],1:[1])");
}

#[test]
fn missing_ratios_fall_back_to_equal_shares() {
    let root = row(Vec::new(), vec![pane(1), pane(2)]);
    let (restored, _) = restore(&single(root));
    assert_eq!(shape(&restored.instances[0]), "Row(1:[1],1:[2])");
}

#[test]
fn extra_ratios_are_ignored() {
    let root = row(vec![2.0, 3.0, 4.0], vec![pane(1), pane(1)]);
    let (restored, _) = restore(&single(root));
    assert_eq!(shape(&restored.instances[0]), "Row(2:[1],3:[1])");
}

#[test]
fn empty_panes_are_dropped_and_the_split_collapses() {
    let root = row(vec![1.0, 1.0], vec![pane(0), pane(2)]);
    let (restored, _) = restore(&single(root));
    assert_eq!(shape(&restored.instances[0]), "[2]");
}

#[test]
fn split_with_a_single_child_collapses() {
    let root = row(vec![1.0], vec![pane(1)]);
    let (restored, _) = restore(&single(root));
    assert_eq!(shape(&restored.instances[0]), "[1]");
}

#[test]
fn split_without_children_is_dropped() {
    let result = Workspace::restore(&single(row(Vec::new(), Vec::new())), &mut |_| SessionId(1));
    assert!(matches!(result, Err(SnapshotError::Empty)));
}

#[test]
fn out_of_range_active_tab_is_clamped() {
    let root = NodeSnapshot::Pane(PaneSnapshot {
        tabs: vec![TabSnapshot::default(); 2],
        active: 9,
    });
    let (restored, _) = restore(&single(root));
    let instance = &restored.instances[0];
    assert_eq!(instance.active().map(|pane| pane.active), Some(1));
}

#[test]
fn out_of_range_active_pane_falls_back_to_the_first() {
    let mut snapshot = single(row(vec![1.0, 1.0], vec![pane(1), pane(1)]));
    snapshot.instances[0].active_pane = 7;
    let (restored, _) = restore(&snapshot);
    let instance = &restored.instances[0];
    assert_eq!(instance.active_pane, instance.root.panes()[0]);
}

#[test]
fn out_of_range_active_instance_is_clamped() {
    let mut snapshot = single(pane(1));
    snapshot.active_instance = 9;
    let (restored, _) = restore(&snapshot);
    assert_eq!(restored.active, 0);
}

#[test]
fn instances_without_tabs_are_skipped() {
    let mut snapshot = single(pane(0));
    snapshot.instances.push(instance_of(pane(2)));
    snapshot.active_instance = 1;
    let (restored, _) = restore(&snapshot);
    assert_eq!(restored.instances.len(), 1);
    assert_eq!(restored.active, 0);
    assert_eq!(restored.tabs().len(), 2);
}

#[test]
fn blank_names_come_back_unnamed() {
    let mut snapshot = single(pane(1));
    snapshot.instances[0].name = "   ".to_owned();
    let (restored, _) = restore(&snapshot);
    assert_eq!(restored.instances[0].name, "");
}
