use std::f32::consts::TAU;

use crate::geometry::Point;

const STEP: f32 = 0.75;
const MAX_SEGMENTS: usize = 64;

fn segments(length: f32) -> usize {
    ((length / STEP).ceil() as usize).clamp(1, MAX_SEGMENTS)
}

fn distance(a: Point, b: Point) -> f32 {
    (b.x - a.x).hypot(b.y - a.y)
}

fn lerp(a: Point, b: Point, t: f32) -> Point {
    Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

pub(super) fn cubic(from: Point, first: Point, second: Point, to: Point) -> Vec<Point> {
    let length = distance(from, first) + distance(first, second) + distance(second, to);
    let count = segments(length);
    (1..=count)
        .map(|step| {
            let t = step as f32 / count as f32;
            let a = lerp(lerp(from, first, t), lerp(first, second, t), t);
            let b = lerp(lerp(first, second, t), lerp(second, to, t), t);
            lerp(a, b, t)
        })
        .collect()
}

pub(super) fn quadratic(from: Point, control: Point, to: Point) -> Vec<Point> {
    let count = segments(distance(from, control) + distance(control, to));
    (1..=count)
        .map(|step| {
            let t = step as f32 / count as f32;
            lerp(lerp(from, control, t), lerp(control, to, t), t)
        })
        .collect()
}

pub(super) struct Arc {
    pub radii: Point,
    pub rotation: f32,
    pub large: bool,
    pub sweep: bool,
}

fn angle(u: Point, v: Point) -> f32 {
    (u.x * v.y - u.y * v.x).atan2(u.x * v.x + u.y * v.y)
}

struct Span {
    center: Point,
    rx: f32,
    ry: f32,
    theta: f32,
    delta: f32,
}

fn span(from: Point, shape: &Arc, to: Point) -> Span {
    let (mut rx, mut ry) = (shape.radii.x.abs(), shape.radii.y.abs());
    let (sin, cos) = shape.rotation.to_radians().sin_cos();
    let dx = (from.x - to.x) / 2.0;
    let dy = (from.y - to.y) / 2.0;
    let x1 = cos * dx + sin * dy;
    let y1 = -sin * dx + cos * dy;
    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        rx *= lambda.sqrt();
        ry *= lambda.sqrt();
    }
    let num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut coef = (num / den).max(0.0).sqrt();
    if shape.large == shape.sweep {
        coef = -coef;
    }
    let cx1 = coef * rx * y1 / ry;
    let cy1 = -coef * ry * x1 / rx;
    let center = Point::new(
        cos * cx1 - sin * cy1 + (from.x + to.x) / 2.0,
        sin * cx1 + cos * cy1 + (from.y + to.y) / 2.0,
    );
    let start = Point::new((x1 - cx1) / rx, (y1 - cy1) / ry);
    let end = Point::new((-x1 - cx1) / rx, (-y1 - cy1) / ry);
    let theta = angle(Point::new(1.0, 0.0), start);
    let mut delta = angle(start, end);
    if !shape.sweep && delta > 0.0 {
        delta -= TAU;
    } else if shape.sweep && delta < 0.0 {
        delta += TAU;
    }
    Span {
        center,
        rx,
        ry,
        theta,
        delta,
    }
}

pub(super) fn arc(from: Point, shape: Arc, to: Point) -> Vec<Point> {
    if from == to {
        return Vec::new();
    }
    if shape.radii.x.abs() == 0.0 || shape.radii.y.abs() == 0.0 {
        return vec![to];
    }
    let Span {
        center,
        rx,
        ry,
        theta,
        delta,
    } = span(from, &shape, to);
    let (sin, cos) = shape.rotation.to_radians().sin_cos();
    let count = segments(delta.abs() * rx.max(ry));
    let mut points: Vec<Point> = (1..count)
        .map(|step| {
            let (s, c) = (theta + delta * step as f32 / count as f32).sin_cos();
            let (ex, ey) = (rx * c, ry * s);
            Point::new(
                cos * ex - sin * ey + center.x,
                sin * ex + cos * ey + center.y,
            )
        })
        .collect();
    points.push(to);
    points
}

pub(in crate::ui::chrome::icons) fn ellipse(center: Point, rx: f32, ry: f32) -> Vec<Point> {
    let count = segments(TAU * rx.max(ry));
    (0..=count)
        .map(|step| {
            let (s, c) = (TAU * step as f32 / count as f32).sin_cos();
            Point::new(center.x + rx * c, center.y + ry * s)
        })
        .collect()
}
