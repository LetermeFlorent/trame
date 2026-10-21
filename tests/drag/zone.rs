use trame::geometry::{Point, Rect};
use trame::ui::drop::DropZone;
use trame::workspace::Edge;

const BODY: Rect = Rect::new(100.0, 50.0, 400.0, 200.0);

fn zone(x: f32, y: f32) -> Option<DropZone> {
    DropZone::at(BODY, Point::new(x, y))
}

#[test]
fn the_middle_adds_a_tab() {
    assert_eq!(zone(300.0, 150.0), Some(DropZone::Center));
}

#[test]
fn bands_near_each_edge_split_that_side() {
    assert_eq!(zone(110.0, 150.0), Some(DropZone::Edge(Edge::Left)));
    assert_eq!(zone(490.0, 150.0), Some(DropZone::Edge(Edge::Right)));
    assert_eq!(zone(300.0, 55.0), Some(DropZone::Edge(Edge::Top)));
    assert_eq!(zone(300.0, 245.0), Some(DropZone::Edge(Edge::Bottom)));
}

#[test]
fn the_closest_edge_wins_in_a_corner() {
    assert_eq!(zone(102.0, 70.0), Some(DropZone::Edge(Edge::Left)));
    assert_eq!(zone(150.0, 52.0), Some(DropZone::Edge(Edge::Top)));
}

#[test]
fn points_outside_or_empty_rects_have_no_zone() {
    assert_eq!(zone(99.0, 150.0), None);
    assert_eq!(zone(500.0, 150.0), None);
    let empty = Rect::new(0.0, 0.0, 0.0, 10.0);
    assert_eq!(DropZone::at(empty, Point::new(0.0, 5.0)), None);
}

#[test]
fn edge_areas_cover_half_the_body() {
    assert_eq!(DropZone::Center.area(BODY), BODY);
    let left = DropZone::Edge(Edge::Left).area(BODY);
    let right = DropZone::Edge(Edge::Right).area(BODY);
    let top = DropZone::Edge(Edge::Top).area(BODY);
    let bottom = DropZone::Edge(Edge::Bottom).area(BODY);
    assert_eq!(left, Rect::new(100.0, 50.0, 200.0, 200.0));
    assert_eq!(right, Rect::new(300.0, 50.0, 200.0, 200.0));
    assert_eq!(top, Rect::new(100.0, 50.0, 400.0, 100.0));
    assert_eq!(bottom, Rect::new(100.0, 150.0, 400.0, 100.0));
}
