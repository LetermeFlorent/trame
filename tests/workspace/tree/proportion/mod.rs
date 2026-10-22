mod pins;

use trame::geometry::Rect;
use trame::workspace::{Axis, Edge, Node, PaneId, PaneLayout, resize_split};

use crate::{column, leaf, pinned_split, row};

const AREA: Rect = Rect::new(0.0, 0.0, 600.0, 100.0);
const TOLERANCE: f32 = 1e-4;

fn flexes_of(node: &Node) -> Vec<f32> {
    match node {
        Node::Split { flexes, .. } => flexes.clone(),
        Node::Pane(_) => Vec::new(),
    }
}

fn pins_of(node: &Node) -> Vec<bool> {
    match node {
        Node::Split { pinned, .. } => pinned.clone(),
        Node::Pane(_) => Vec::new(),
    }
}

fn assert_flexes(node: &Node, expected: &[f32]) {
    let flexes = flexes_of(node);
    assert_eq!(flexes.len(), expected.len());
    for (flex, wanted) in flexes.iter().zip(expected) {
        assert!(
            (flex - wanted).abs() < TOLERANCE,
            "{flexes:?} != {expected:?}"
        );
    }
}

fn drag(root: &mut Node, index: usize, delta: f32) {
    let divider = PaneLayout::compute(root, AREA, 4.0).dividers.remove(index);
    resize_split(root, &divider, delta);
}

fn pinned_row(flexes: Vec<f32>, pinned: Vec<bool>) -> Node {
    let children = (1..=flexes.len() as u32).map(leaf).collect();
    pinned_split(Axis::Row, children, flexes, pinned)
}

#[test]
fn splitting_the_right_pane_of_two_columns_gives_three_thirds() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    assert!(root.split(PaneId(2), PaneId(3), Edge::Right));
    assert_flexes(&root, &[2.0 / 3.0; 3]);
    assert_eq!(pins_of(&root), vec![false; 3]);
}

#[test]
fn splitting_again_keeps_every_pane_equal() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    root.split(PaneId(2), PaneId(3), Edge::Right);
    root.split(PaneId(3), PaneId(4), Edge::Left);
    assert_flexes(&root, &[0.5; 4]);
}

#[test]
fn too_little_free_space_falls_back_to_halving_the_target() {
    let mut root = pinned_row(vec![5.0, 0.1, 5.0], vec![true, false, true]);
    assert!(root.split(PaneId(2), PaneId(4), Edge::Right));
    assert_flexes(&root, &[5.0, 0.05, 0.05, 5.0]);
    assert_eq!(pins_of(&root), vec![true, false, false, true]);
}

#[test]
fn splitting_along_the_other_axis_nests_a_fresh_unpinned_split() {
    let mut root = pinned_row(vec![1.0, 3.0], vec![true, false]);
    assert!(root.split(PaneId(2), PaneId(3), Edge::Bottom));
    assert_eq!(flexes_of(&root), vec![1.0, 3.0]);
    assert_eq!(pins_of(&root), vec![true, false]);
    let Node::Split { children, .. } = &root else {
        panic!("root stays a split");
    };
    assert_eq!(children[1], column(vec![leaf(2), leaf(3)]));
}
