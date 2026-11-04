use trame::workspace::{Axis, Edge};

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
