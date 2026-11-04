use trame::geometry::{Point, Rect};

const BOX: Rect = Rect::new(10.0, 20.0, 100.0, 50.0);

#[test]
fn contains_includes_the_top_left_edge_and_excludes_the_bottom_right() {
    assert!(BOX.contains(Point::new(10.0, 20.0)));
    assert!(BOX.contains(Point::new(109.5, 69.5)));
    assert!(!BOX.contains(Point::new(110.0, 30.0)));
    assert!(!BOX.contains(Point::new(50.0, 70.0)));
    assert!(!BOX.contains(Point::new(9.9, 30.0)));
}

#[test]
fn union_covers_both_rects() {
    let other = Rect::new(0.0, 60.0, 20.0, 40.0);
    assert_eq!(BOX.union(other), Rect::new(0.0, 20.0, 110.0, 80.0));
}

#[test]
fn inset_shrinks_every_side_and_never_goes_negative() {
    assert_eq!(BOX.inset(5.0), Rect::new(15.0, 25.0, 90.0, 40.0));
    assert!(BOX.inset(60.0).is_empty());
    assert_eq!(BOX.inset(60.0).width, 0.0);
}

#[test]
fn split_left_cuts_at_the_width_and_clamps() {
    let (left, rest) = BOX.split_left(30.0);
    assert_eq!(left, Rect::new(10.0, 20.0, 30.0, 50.0));
    assert_eq!(rest, Rect::new(40.0, 20.0, 70.0, 50.0));
    let (all, none) = BOX.split_left(500.0);
    assert_eq!(all, BOX);
    assert_eq!(none.width, 0.0);
    let (nothing, whole) = BOX.split_left(-5.0);
    assert_eq!(nothing.width, 0.0);
    assert_eq!(whole, BOX);
}

#[test]
fn split_top_cuts_at_the_height_and_clamps() {
    let (top, rest) = BOX.split_top(10.0);
    assert_eq!(top, Rect::new(10.0, 20.0, 100.0, 10.0));
    assert_eq!(rest, Rect::new(10.0, 30.0, 100.0, 40.0));
    let (all, none) = BOX.split_top(500.0);
    assert_eq!(all, BOX);
    assert_eq!(none.height, 0.0);
}

#[test]
fn snapped_rounds_the_edges_rather_than_the_size() {
    let rect = Rect::new(0.4, 0.6, 10.4, 10.4);
    assert_eq!(rect.snapped(), Rect::new(0.0, 1.0, 11.0, 10.0));
}
