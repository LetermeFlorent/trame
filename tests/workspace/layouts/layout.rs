use trame::geometry::Rect;
use trame::workspace::{Axis, PaneId, PaneLayout};

use crate::{column, leaf, row, weighted};

const AREA: Rect = Rect::new(0.0, 0.0, 200.0, 100.0);

fn rect_of(layout: &PaneLayout, pane: u32) -> Rect {
    layout.rect_of(PaneId(pane)).expect("pane is placed")
}

#[test]
fn a_lone_pane_fills_the_whole_area() {
    let layout = PaneLayout::compute(&leaf(1), AREA, 4.0);
    assert_eq!(layout.panes, vec![(PaneId(1), AREA)]);
    assert!(layout.dividers.is_empty());
}

#[test]
fn two_panes_split_right_share_the_width_around_the_gap() {
    let layout = PaneLayout::compute(&row(vec![leaf(1), leaf(2)]), AREA, 4.0);
    assert_eq!(rect_of(&layout, 1), Rect::new(0.0, 0.0, 98.0, 100.0));
    assert_eq!(rect_of(&layout, 2), Rect::new(102.0, 0.0, 98.0, 100.0));
}

#[test]
fn split_panes_do_not_overlap_and_fill_the_area() {
    let layout = PaneLayout::compute(&row(vec![leaf(1), leaf(2)]), AREA, 4.0);
    let (first, second) = (rect_of(&layout, 1), rect_of(&layout, 2));
    assert!(first.intersect(second).is_empty());
    assert_eq!(first.x, AREA.x);
    assert_eq!(second.right(), AREA.right());
    assert_eq!(first.right() + 4.0, second.x);
    assert_eq!(first.width + 4.0 + second.width, AREA.width);
}

#[test]
fn a_column_stacks_panes_top_to_bottom() {
    let layout = PaneLayout::compute(&column(vec![leaf(1), leaf(2)]), AREA, 4.0);
    assert_eq!(rect_of(&layout, 1), Rect::new(0.0, 0.0, 200.0, 48.0));
    assert_eq!(rect_of(&layout, 2), Rect::new(0.0, 52.0, 200.0, 48.0));
}

#[test]
fn the_divider_sits_in_the_gap_between_two_panes() {
    let layout = PaneLayout::compute(&row(vec![leaf(1), leaf(2)]), AREA, 4.0);
    let divider = &layout.dividers[0];
    assert_eq!(layout.dividers.len(), 1);
    assert_eq!(divider.rect, Rect::new(98.0, 0.0, 4.0, 100.0));
    assert_eq!((divider.axis, divider.index), (Axis::Row, 0));
    assert_eq!(divider.extent, 196.0);
    assert!(divider.path.is_empty());
}

#[test]
fn flexes_weight_the_sizes() {
    let root = weighted(Axis::Row, vec![leaf(1), leaf(2)], vec![1.0, 3.0]);
    let layout = PaneLayout::compute(&root, AREA, 0.0);
    assert_eq!(rect_of(&layout, 1).width, 50.0);
    assert_eq!(rect_of(&layout, 2).width, 150.0);
}

#[test]
fn rounding_never_leaves_a_hole_between_equal_panes() {
    let root = row(vec![leaf(1), leaf(2), leaf(3)]);
    let layout = PaneLayout::compute(&root, Rect::new(0.0, 0.0, 100.0, 10.0), 0.0);
    let rects: Vec<Rect> = layout.panes.iter().map(|(_, rect)| *rect).collect();
    assert_eq!(rects[0].x, 0.0);
    assert_eq!(rects[0].right(), rects[1].x);
    assert_eq!(rects[1].right(), rects[2].x);
    assert_eq!(rects[2].right(), 100.0);
}

#[test]
fn nested_splits_stay_inside_their_parent_cell() {
    let root = row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]);
    let layout = PaneLayout::compute(&root, AREA, 0.0);
    assert_eq!(rect_of(&layout, 1), Rect::new(0.0, 0.0, 100.0, 100.0));
    assert_eq!(rect_of(&layout, 2), Rect::new(100.0, 0.0, 100.0, 50.0));
    assert_eq!(rect_of(&layout, 3), Rect::new(100.0, 50.0, 100.0, 50.0));
    let kinds: Vec<(Vec<usize>, Axis)> = layout
        .dividers
        .iter()
        .map(|divider| (divider.path.clone(), divider.axis))
        .collect();
    assert_eq!(kinds, vec![(vec![], Axis::Row), (vec![1], Axis::Column)]);
}
