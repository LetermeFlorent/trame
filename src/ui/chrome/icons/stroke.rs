use super::Pen;
use super::outline::Shape;
use crate::geometry::{Point, Rect};

const VIEW: f32 = 24.0;
const LUCIDE_STROKE: f32 = 2.0;
const AXIS: f32 = 0.001;

fn crisp(value: f32, width: f32) -> f32 {
    (value - width / 2.0).round() + width / 2.0
}

pub(super) fn draw(pen: &mut Pen, shapes: &[Shape], rect: Rect) {
    let scale = rect.width.min(rect.height) / VIEW;
    let width = (scale * LUCIDE_STROKE).max(pen.stroke);
    let center = rect.center();
    let map = |point: Point| {
        Point::new(
            center.x + (point.x - VIEW / 2.0) * scale,
            center.y + (point.y - VIEW / 2.0) * scale,
        )
    };
    for shape in shapes {
        match shape {
            Shape::Stroke(points) => {
                let points: Vec<Point> = points.iter().copied().map(map).collect();
                polyline(pen, &snap(&points, width), width);
            }
            Shape::Ring { center, radius } => {
                let at = map(*center);
                let reach = radius * scale + width / 2.0;
                let ring = Rect::new(at.x - reach, at.y - reach, reach * 2.0, reach * 2.0);
                pen.scene.border(ring, pen.color, reach, width);
            }
            Shape::Frame { rect, radius } => {
                let corner = map(Point::new(rect.x, rect.y));
                let far = map(Point::new(rect.right(), rect.bottom()));
                let frame = Rect::from_edges(corner.x, corner.y, far.x, far.y);
                let outer = frame.inset(-width / 2.0).snapped();
                pen.scene
                    .border(outer, pen.color, radius * scale + width / 2.0, width);
            }
        }
    }
}

fn snap(points: &[Point], width: f32) -> Vec<Point> {
    let neighbours = |index: usize| {
        let before = index.checked_sub(1).and_then(|before| points.get(before));
        before.into_iter().chain(points.get(index + 1))
    };
    points
        .iter()
        .enumerate()
        .map(|(index, &point)| {
            let mut snapped = point;
            for other in neighbours(index).filter(|other| **other != point) {
                if (other.x - point.x).abs() < AXIS {
                    snapped.x = crisp(point.x, width);
                }
                if (other.y - point.y).abs() < AXIS {
                    snapped.y = crisp(point.y, width);
                }
            }
            snapped
        })
        .collect()
}

fn polyline(pen: &mut Pen, points: &[Point], width: f32) {
    for pair in points.windows(2) {
        pen.scene.line(pair[0], pair[1], width, pen.color);
    }
}
