use trame::geometry::Rect;
use trame::workspace::{Axis, Divider, MIN_PANE, Node, PaneLayout, resize_split};

use crate::{column, leaf, row, weighted};

const AREA: Rect = Rect::new(0.0, 0.0, 200.0, 100.0);

fn dividers(root: &Node, area: Rect) -> Vec<Divider> {
    PaneLayout::compute(root, area, 4.0).dividers
}

fn widths(root: &Node, area: Rect) -> Vec<f32> {
    let layout = PaneLayout::compute(root, area, 4.0);
    layout.panes.iter().map(|(_, rect)| rect.width).collect()
}

fn flex_sum(root: &Node) -> f32 {
    match root {
        Node::Split { flexes, .. } => flexes.iter().sum(),
        Node::Pane(_) => 0.0,
    }
}

#[test]
fn dragging_the_divider_right_grows_the_first_pane() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    let divider = dividers(&root, AREA).remove(0);
    resize_split(&mut root, &divider, 10.0);
    assert_eq!(widths(&root, AREA), vec![108.0, 88.0]);
}

#[test]
fn dragging_the_divider_left_shrinks_the_first_pane() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    let divider = dividers(&root, AREA).remove(0);
    resize_split(&mut root, &divider, -10.0);
    assert_eq!(widths(&root, AREA), vec![88.0, 108.0]);
}

#[test]
fn dragging_far_right_stops_at_the_minimum_width_of_the_second_pane() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    let divider = dividers(&root, AREA).remove(0);
    resize_split(&mut root, &divider, 10_000.0);
    assert_eq!(widths(&root, AREA), vec![116.0, MIN_PANE]);
}

#[test]
fn dragging_far_left_stops_at_the_minimum_width_of_the_first_pane() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    let divider = dividers(&root, AREA).remove(0);
    resize_split(&mut root, &divider, -10_000.0);
    assert_eq!(widths(&root, AREA), vec![MIN_PANE, 116.0]);
}

#[test]
fn resizing_keeps_the_total_flex() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    let divider = dividers(&root, AREA).remove(0);
    resize_split(&mut root, &divider, 30.0);
    assert!((flex_sum(&root) - 2.0).abs() < 1e-5);
}

#[test]
fn an_area_too_small_for_two_minimum_panes_is_left_alone() {
    let area = Rect::new(0.0, 0.0, 100.0, 100.0);
    let mut root = row(vec![leaf(1), leaf(2)]);
    let divider = dividers(&root, area).remove(0);
    resize_split(&mut root, &divider, 30.0);
    assert_eq!(root, row(vec![leaf(1), leaf(2)]));
}

#[test]
fn only_the_two_panes_around_the_divider_change() {
    let area = Rect::new(0.0, 0.0, 300.0, 100.0);
    let mut root = row(vec![leaf(1), leaf(2), leaf(3)]);
    let layout = PaneLayout::compute(&root, area, 0.0);
    let divider = layout.dividers[1].clone();
    resize_split(&mut root, &divider, 10.0);
    let sizes: Vec<f32> = PaneLayout::compute(&root, area, 0.0)
        .panes
        .iter()
        .map(|(_, rect)| rect.width)
        .collect();
    assert_eq!(sizes, vec![100.0, 110.0, 90.0]);
}

#[test]
fn a_nested_divider_resizes_its_own_split_only() {
    let area = Rect::new(0.0, 0.0, 200.0, 200.0);
    let mut root = row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]);
    let divider = dividers(&root, area).remove(1);
    assert_eq!(divider.path, vec![1]);
    resize_split(&mut root, &divider, 10.0);
    let Node::Split {
        flexes, children, ..
    } = &root
    else {
        panic!("root stays a split");
    };
    assert_eq!(flexes, &vec![1.0, 1.0]);
    let Node::Split { flexes: inner, .. } = &children[1] else {
        panic!("nested split stays a split");
    };
    assert!(inner[0] > inner[1]);
}

#[test]
fn a_divider_with_a_stale_path_changes_nothing() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    let mut divider = dividers(&root, AREA).remove(0);
    divider.path = vec![4];
    resize_split(&mut root, &divider, 10.0);
    assert_eq!(root, row(vec![leaf(1), leaf(2)]));
}

#[test]
fn a_divider_past_the_last_child_changes_nothing() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    let mut divider = dividers(&root, AREA).remove(0);
    divider.index = 1;
    resize_split(&mut root, &divider, 10.0);
    assert_eq!(root, row(vec![leaf(1), leaf(2)]));
}

#[test]
fn a_divider_without_extent_changes_nothing() {
    let mut root = weighted(Axis::Row, vec![leaf(1), leaf(2)], vec![1.0, 1.0]);
    let mut divider = dividers(&root, AREA).remove(0);
    divider.extent = 0.0;
    resize_split(&mut root, &divider, 10.0);
    assert_eq!(root, row(vec![leaf(1), leaf(2)]));
}
