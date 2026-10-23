use super::Pen;
use crate::geometry::Point;

const PENCIL_TIP: f32 = -0.3;
const PENCIL_NECK: f32 = -0.18;
const PENCIL_END: f32 = 0.3;
const PENCIL_SIDE: f32 = 0.07;

pub(super) fn pencil(pen: &mut Pen, center: Point, size: f32) {
    let at = |along: f32, side: f32| {
        Point::new(
            center.x + size * (along + side),
            center.y + size * (side - along),
        )
    };
    let sides = |along: f32| [PENCIL_SIDE, -PENCIL_SIDE].map(|side| at(along, side));
    let tip = at(PENCIL_TIP, 0.0);
    let [neck_a, neck_b] = sides(PENCIL_NECK);
    let [end_a, end_b] = sides(PENCIL_END);
    pen.line(tip, neck_a);
    pen.line(tip, neck_b);
    pen.line(neck_a, neck_b);
    pen.line(neck_a, end_a);
    pen.line(neck_b, end_b);
    pen.line(end_a, end_b);
}
