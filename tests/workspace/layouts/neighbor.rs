use trame::geometry::Rect;
use trame::workspace::{Axis, Edge, Node, PaneId, PaneLayout};

use crate::{column, leaf, row, weighted};

const AREA: Rect = Rect::new(0.0, 0.0, 200.0, 100.0);

fn grid() -> Node {
    column(vec![
        row(vec![leaf(1), leaf(2)]),
        row(vec![leaf(3), leaf(4)]),
    ])
}

#[test]
fn neighbors_in_a_grid_follow_each_edge() {
    let layout = PaneLayout::compute(&grid(), AREA, 4.0);
    assert_eq!(layout.neighbor(PaneId(1), Edge::Right), Some(PaneId(2)));
    assert_eq!(layout.neighbor(PaneId(1), Edge::Bottom), Some(PaneId(3)));
    assert_eq!(layout.neighbor(PaneId(4), Edge::Left), Some(PaneId(3)));
    assert_eq!(layout.neighbor(PaneId(4), Edge::Top), Some(PaneId(2)));
}

#[test]
fn no_neighbor_exists_past_the_outer_edges() {
    let layout = PaneLayout::compute(&grid(), AREA, 4.0);
    assert_eq!(layout.neighbor(PaneId(1), Edge::Left), None);
    assert_eq!(layout.neighbor(PaneId(1), Edge::Top), None);
    assert_eq!(layout.neighbor(PaneId(4), Edge::Right), None);
    assert_eq!(layout.neighbor(PaneId(4), Edge::Bottom), None);
}

#[test]
fn neighbor_prefers_the_pane_whose_center_is_closest() {
    let tall_top = weighted(Axis::Column, vec![leaf(2), leaf(3)], vec![3.0, 1.0]);
    let layout = PaneLayout::compute(&row(vec![leaf(1), tall_top]), AREA, 4.0);
    assert_eq!(layout.neighbor(PaneId(1), Edge::Right), Some(PaneId(2)));
    let tall_bottom = weighted(Axis::Column, vec![leaf(2), leaf(3)], vec![1.0, 3.0]);
    let layout = PaneLayout::compute(&row(vec![leaf(1), tall_bottom]), AREA, 4.0);
    assert_eq!(layout.neighbor(PaneId(1), Edge::Right), Some(PaneId(3)));
}

#[test]
fn neighbor_of_an_unplaced_or_lone_pane_is_none() {
    let layout = PaneLayout::compute(&leaf(1), AREA, 4.0);
    assert_eq!(layout.neighbor(PaneId(1), Edge::Right), None);
    assert_eq!(layout.neighbor(PaneId(7), Edge::Right), None);
    assert_eq!(layout.rect_of(PaneId(7)), None);
}
