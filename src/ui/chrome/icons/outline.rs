use std::sync::OnceLock;

use super::lucide::{Element, Glyph};
use super::path::{ellipse, flatten};
use crate::geometry::{Point, Rect};

#[derive(Debug, PartialEq)]
pub(super) enum Shape {
    Stroke(Vec<Point>),
    Ring { center: Point, radius: f32 },
    Frame { rect: Rect, radius: f32 },
}

fn shapes(element: &Element) -> Vec<Shape> {
    match *element {
        Element::Path(d) => flatten(d).into_iter().map(Shape::Stroke).collect(),
        Element::Circle(x, y, radius) => vec![Shape::Ring {
            center: Point::new(x, y),
            radius,
        }],
        Element::Rect(x, y, width, height, radius) => vec![Shape::Frame {
            rect: Rect::new(x, y, width, height),
            radius: radius.min(width / 2.0).min(height / 2.0),
        }],
        Element::Line(x1, y1, x2, y2) => {
            vec![Shape::Stroke(vec![Point::new(x1, y1), Point::new(x2, y2)])]
        }
        Element::Polyline(points) => vec![Shape::Stroke(
            points
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&[x, y]| Point::new(x, y))
                .collect(),
        )],
        Element::Ellipse(x, y, rx, ry) => vec![Shape::Stroke(ellipse(Point::new(x, y), rx, ry))],
    }
}

pub(super) fn build(glyph: Glyph) -> Vec<Shape> {
    glyph.source().iter().flat_map(shapes).collect()
}

pub(super) fn outline(glyph: Glyph) -> &'static [Shape] {
    static CACHE: OnceLock<Vec<Vec<Shape>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Glyph::ALL.into_iter().map(build).collect());
    &cache[glyph as usize]
}
