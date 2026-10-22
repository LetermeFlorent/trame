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
fn a_dragged_divider_pins_both_neighbours() {
    let mut root = row(vec![leaf(1), leaf(2), leaf(3)]);
    drag(&mut root, 0, 30.0);
    assert_eq!(pins_of(&root), vec![true, true, false]);
}

#[test]
fn a_later_split_keeps_pinned_sizes_and_shares_the_rest() {
    let mut root = row(vec![leaf(1), leaf(2), leaf(3)]);
    drag(&mut root, 0, 30.0);
    let before = flexes_of(&root);
    assert!(root.split(PaneId(3), PaneId(4), Edge::Right));
    let after = flexes_of(&root);
    assert_eq!(after[..2], before[..2]);
    assert!((after[2] - before[2] / 2.0).abs() < TOLERANCE);
    assert!((after[3] - before[2] / 2.0).abs() < TOLERANCE);
    assert_eq!(pins_of(&root), vec![true, true, false, false]);
}

#[test]
fn splitting_a_pinned_pane_unpins_it_and_shares_with_the_free_panes() {
    let mut root = pinned_row(vec![1.0, 2.0, 1.0], vec![true, true, false]);
    assert!(root.split(PaneId(2), PaneId(4), Edge::Right));
    assert_flexes(&root, &[1.0; 4]);
    assert_eq!(pins_of(&root), vec![true, false, false, false]);
}

#[test]
fn when_every_other_pane_is_pinned_the_target_and_new_pane_split_the_rest() {
    let mut root = pinned_row(vec![1.0, 2.0, 1.0], vec![true, false, true]);
    assert!(root.split(PaneId(2), PaneId(4), Edge::Left));
    assert_flexes(&root, &[1.0, 1.0, 1.0, 1.0]);
    assert_eq!(pins_of(&root), vec![true, false, false, true]);
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

#[test]
fn removing_a_pane_keeps_the_pins_of_the_others() {
    let mut root = pinned_row(vec![1.0, 2.0, 3.0], vec![true, false, true]);
    assert!(root.remove(PaneId(2)));
    assert_eq!(flexes_of(&root), vec![1.0, 3.0]);
    assert_eq!(pins_of(&root), vec![true, true]);
}

#[test]
fn flattening_a_nested_split_keeps_the_inner_pins() {
    let inner = pinned_split(
        Axis::Row,
        vec![leaf(2), leaf(3), leaf(4)],
        vec![1.0, 1.0, 1.0],
        vec![true, false, false],
    );
    let mut root = row(vec![leaf(1), inner]);
    assert!(root.remove(PaneId(4)));
    assert_eq!(pins_of(&root), vec![false, true, false]);
    assert_eq!(flexes_of(&root).len(), 3);
}

#[test]
fn a_pin_list_of_the_wrong_length_counts_as_unpinned() {
    let mut root = pinned_row(vec![1.0, 1.0], Vec::new());
    assert!(root.split(PaneId(2), PaneId(3), Edge::Right));
    assert_flexes(&root, &[2.0 / 3.0; 3]);
    assert_eq!(pins_of(&root), vec![false; 3]);
}

#[test]
fn a_divider_press_without_movement_pins_nothing() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    drag(&mut root, 0, 0.0);
    assert_eq!(pins_of(&root), vec![false, false]);
}
