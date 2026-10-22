use trame::geometry::{Point, Rect};
use trame::session::SessionId;
use trame::workspace::{Divider, Edge, MIN_PANE, Node, PaneLayout, Workspace, resized};

const AREA: Rect = Rect::new(0.0, 0.0, 1001.0, 600.0);
const GAP: f32 = 1.0;

fn split(edge: Edge) -> Node {
    let mut workspace = Workspace::new();
    workspace.add_instance(None, SessionId(1));
    workspace.split_active(SessionId(2), edge);
    workspace
        .active_instance()
        .expect("an instance")
        .root
        .clone()
}

fn sizes(root: &Node, measure: fn(&Rect) -> f32) -> Vec<f32> {
    PaneLayout::compute(root, AREA, GAP)
        .panes
        .iter()
        .map(|(_, rect)| measure(rect))
        .collect()
}

fn widths(root: &Node) -> Vec<f32> {
    sizes(root, |rect| rect.width)
}

fn divider(root: &Node) -> Divider {
    PaneLayout::compute(root, AREA, GAP).dividers[0].clone()
}

#[test]
fn delta_follows_the_divider_axis() {
    let row = divider(&split(Edge::Right));
    let column = divider(&split(Edge::Bottom));
    let from = Point::new(10.0, 20.0);
    let to = Point::new(40.0, 5.0);
    assert_eq!(row.delta(from, to), 30.0);
    assert_eq!(column.delta(from, to), -15.0);
}

#[test]
fn moving_the_divider_resizes_both_neighbours() {
    let root = split(Edge::Right);
    let moved = resized(&root, &divider(&root), 100.0);
    assert_eq!(widths(&moved), vec![600.0, 400.0]);
}

#[test]
fn resizing_from_the_press_state_does_not_drift() {
    let root = split(Edge::Right);
    let line = divider(&root);
    let mut current = root.clone();
    for delta in [40.0, 120.0, -30.0, 75.0] {
        current = resized(&root, &line, delta);
    }
    assert_eq!(current, resized(&root, &line, 75.0));
    assert_eq!(resized(&root, &line, 0.0), root);
}

#[test]
fn panes_never_shrink_below_the_minimum() {
    let root = split(Edge::Right);
    let line = divider(&root);
    let narrow = widths(&resized(&root, &line, -2000.0));
    let wide = widths(&resized(&root, &line, 2000.0));
    assert!((narrow[0] - MIN_PANE).abs() <= 1.0);
    assert!((wide[1] - MIN_PANE).abs() <= 1.0);
}

#[test]
fn vertical_splits_resize_heights() {
    let root = split(Edge::Bottom);
    let moved = sizes(&resized(&root, &divider(&root), -100.0), |rect| rect.height);
    assert!(moved[0] < moved[1]);
}
