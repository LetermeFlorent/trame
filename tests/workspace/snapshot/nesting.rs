use trame::session::SessionId;
use trame::workspace::{Axis, Edge, Node, Snapshot, Workspace};

use super::{pane, restore, shape, single, snapshot_of, split, split_row, through_json};
use crate::workspace_with;

fn root_pins(workspace: &Workspace) -> &Vec<bool> {
    let Node::Split { pinned, .. } = &workspace.instances[0].root else {
        panic!("root stays a split");
    };
    pinned
}

fn nested() -> Workspace {
    let mut workspace = workspace_with(&[1]);
    workspace.split_active(SessionId(2), Edge::Right);
    workspace.split_active(SessionId(3), Edge::Bottom);
    workspace.split_active(SessionId(4), Edge::Right);
    workspace.add_tab(SessionId(5));
    workspace
}

#[test]
fn nested_splits_keep_their_shape() {
    let original = nested();
    let (restored, _) = restore(&snapshot_of(&original, &[]));
    assert_eq!(shape(&restored.instances[0]), shape(&original.instances[0]));
    assert_eq!(restored.instances[0].root.panes().len(), 4);
    assert_eq!(restored.tabs().len(), 5);
}

#[test]
fn nested_splits_alternate_axes() {
    let (restored, _) = restore(&snapshot_of(&nested(), &[]));
    let Node::Split { axis, children, .. } = &restored.instances[0].root else {
        panic!("the root is a split");
    };
    assert_eq!(*axis, Axis::Row);
    let Node::Split { axis: inner, .. } = &children[1] else {
        panic!("the second child is a split");
    };
    assert_eq!(*inner, Axis::Column);
}

#[test]
fn nested_splits_survive_json() {
    let snapshot = snapshot_of(&nested(), &[(1, "/a"), (4, "/b")]);
    assert_eq!(through_json(&snapshot), snapshot);
}

#[test]
fn the_focused_pane_is_the_last_one_created() {
    let (restored, _) = restore(&snapshot_of(&nested(), &[]));
    let instance = &restored.instances[0];
    let order = instance.root.panes();
    assert_eq!(instance.active_pane, order[order.len() - 1]);
}

#[test]
fn ratios_are_kept() {
    let root = split_row(vec![3.0, 1.0], vec![pane(1), pane(1)]);
    let (restored, _) = restore(&single(root));
    assert_eq!(shape(&restored.instances[0]), "Row(3:[1],1:[1])");
}

#[test]
fn deep_trees_are_restored_pane_by_pane() {
    let inner = split_row(vec![1.0, 2.0], vec![pane(1), pane(2)]);
    let root = split(
        Axis::Column,
        vec![1.0, 1.0],
        Vec::new(),
        vec![pane(3), inner],
    );
    let (restored, seen) = restore(&single(root));
    assert_eq!(
        shape(&restored.instances[0]),
        "Column(1:[3],1:Row(1:[1],2:[2]))"
    );
    assert_eq!(seen.len(), 6);
}

#[test]
fn pins_survive_a_round_trip() {
    let root = split(
        Axis::Row,
        vec![1.0, 1.0],
        vec![true, false],
        vec![pane(1), pane(1)],
    );
    let (restored, _) = restore(&single(root));
    assert_eq!(root_pins(&restored), &vec![true, false]);
}

#[test]
fn an_old_layout_without_pins_loads_as_unpinned() {
    let json = r#"{"version":1,"sidebar_open":true,"font_size":14.0,"active_instance":0,
        "instances":[{"name":"a","active_pane":0,"root":{"Split":{"axis":"Row","ratios":[1.0,2.0],
        "children":[{"Pane":{"tabs":[{}],"active":0}},{"Pane":{"tabs":[{}],"active":0}}]}}}]}"#;
    let snapshot = Snapshot::from_json(json).expect("old layouts still load");
    let (restored, _) = restore(&snapshot);
    assert_eq!(root_pins(&restored), &vec![false, false]);
}

#[test]
fn a_pin_list_of_the_wrong_length_loads_as_unpinned() {
    let root = split(
        Axis::Row,
        vec![1.0, 1.0],
        vec![true],
        vec![pane(1), pane(1)],
    );
    let (restored, _) = restore(&single(root));
    assert_eq!(root_pins(&restored), &vec![false, false]);
}
