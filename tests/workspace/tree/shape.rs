use trame::workspace::{Axis, Edge, PaneId};

use crate::{column, leaf, row, weighted};

#[test]
fn edges_map_to_an_axis_and_a_side() {
    assert_eq!(Edge::Left.axis(), Axis::Row);
    assert_eq!(Edge::Right.axis(), Axis::Row);
    assert_eq!(Edge::Top.axis(), Axis::Column);
    assert_eq!(Edge::Bottom.axis(), Axis::Column);
    assert!(Edge::Left.is_leading());
    assert!(Edge::Top.is_leading());
    assert!(!Edge::Right.is_leading());
    assert!(!Edge::Bottom.is_leading());
}

#[test]
fn panes_are_listed_in_reading_order() {
    let root = row(vec![leaf(3), column(vec![leaf(1), leaf(2)]), leaf(4)]);
    assert_eq!(
        root.panes(),
        vec![PaneId(3), PaneId(1), PaneId(2), PaneId(4)]
    );
}

#[test]
fn contains_finds_nested_panes_and_misses_absent_ones() {
    let root = row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]);
    assert!(root.contains(PaneId(3)));
    assert!(!root.contains(PaneId(9)));
    assert!(leaf(5).contains(PaneId(5)));
}

#[test]
fn splitting_right_or_bottom_puts_the_new_pane_after_the_target() {
    let mut root = leaf(1);
    assert!(root.split(PaneId(1), PaneId(2), Edge::Right));
    assert_eq!(root, row(vec![leaf(1), leaf(2)]));
    let mut root = leaf(1);
    assert!(root.split(PaneId(1), PaneId(2), Edge::Bottom));
    assert_eq!(root, column(vec![leaf(1), leaf(2)]));
}

#[test]
fn splitting_left_or_top_puts_the_new_pane_before_the_target() {
    let mut root = leaf(1);
    assert!(root.split(PaneId(1), PaneId(2), Edge::Left));
    assert_eq!(root, row(vec![leaf(2), leaf(1)]));
    let mut root = leaf(1);
    assert!(root.split(PaneId(1), PaneId(2), Edge::Top));
    assert_eq!(root, column(vec![leaf(2), leaf(1)]));
}

#[test]
fn splitting_an_unknown_pane_fails_and_keeps_the_tree() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    assert!(!root.split(PaneId(9), PaneId(3), Edge::Right));
    assert_eq!(root, row(vec![leaf(1), leaf(2)]));
}

#[test]
fn splitting_along_the_same_axis_joins_the_row_and_halves_the_target_share() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    assert!(root.split(PaneId(2), PaneId(3), Edge::Right));
    let expected = weighted(
        Axis::Row,
        vec![leaf(1), leaf(2), leaf(3)],
        vec![1.0, 0.5, 0.5],
    );
    assert_eq!(root, expected);
}

#[test]
fn splitting_along_the_other_axis_nests_a_new_split() {
    let mut root = row(vec![leaf(1), leaf(2)]);
    assert!(root.split(PaneId(2), PaneId(3), Edge::Bottom));
    assert_eq!(root, row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]));
}

#[test]
fn splitting_a_deeply_nested_pane_reaches_it() {
    let mut root = row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]);
    assert!(root.split(PaneId(3), PaneId(4), Edge::Left));
    let nested = column(vec![leaf(2), row(vec![leaf(4), leaf(3)])]);
    assert_eq!(root, row(vec![leaf(1), nested]));
}
