use trame::workspace::{Axis, Node, PaneId};

use crate::{column, leaf, row, weighted};

#[test]
fn removing_the_last_pane_of_a_lone_leaf_is_refused() {
    let mut root = leaf(1);
    assert!(!root.remove(PaneId(1)));
    assert_eq!(root, leaf(1));
}

#[test]
fn removing_an_unknown_pane_fails_and_keeps_the_tree() {
    let mut root = row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]);
    assert!(!root.remove(PaneId(9)));
    assert_eq!(root, row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]));
}

#[test]
fn removing_one_of_two_children_collapses_the_split_to_the_survivor() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    assert!(root.remove(PaneId(1)));
    assert_eq!(root, leaf(2));
}

#[test]
fn removing_a_nested_pane_collapses_the_inner_split() {
    let mut root = row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]);
    assert!(root.remove(PaneId(2)));
    assert_eq!(root, row(vec![leaf(1), leaf(3)]));
}

#[test]
fn removing_a_pane_merges_a_same_axis_grandchild_into_its_parent() {
    let inner = column(vec![leaf(2), row(vec![leaf(3), leaf(4)])]);
    let mut root = row(vec![leaf(1), inner]);
    assert!(root.remove(PaneId(2)));
    let expected = weighted(
        Axis::Row,
        vec![leaf(1), leaf(3), leaf(4)],
        vec![1.0, 0.5, 0.5],
    );
    assert_eq!(root, expected);
}

#[test]
fn removing_a_pane_keeps_the_flex_of_the_others() {
    let mut root = weighted(
        Axis::Row,
        vec![leaf(1), leaf(2), leaf(3)],
        vec![1.0, 2.0, 3.0],
    );
    assert!(root.remove(PaneId(2)));
    assert_eq!(
        root,
        weighted(Axis::Row, vec![leaf(1), leaf(3)], vec![1.0, 3.0])
    );
}

#[test]
fn node_mut_follows_a_path_of_child_indexes() {
    let mut root = row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]);
    assert_eq!(root.node_mut(&[1, 0]), Some(&mut leaf(2)));
    assert_eq!(root.node_mut(&[1, 5]), None);
    assert_eq!(root.node_mut(&[0, 0]), None);
    assert!(matches!(root.node_mut(&[]), Some(Node::Split { .. })));
}
