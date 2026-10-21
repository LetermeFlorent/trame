use crate::geometry::{Point, Rect};
use crate::workspace::Edge;

pub const EDGE_BAND: f32 = 0.25;
const HALF: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropZone {
    Center,
    Edge(Edge),
}

impl DropZone {
    pub fn at(rect: Rect, point: Point) -> Option<Self> {
        if rect.is_empty() || !rect.contains(point) {
            return None;
        }
        let left = (point.x - rect.x) / rect.width;
        let top = (point.y - rect.y) / rect.height;
        let nearest = [
            (left, Edge::Left),
            (1.0 - left, Edge::Right),
            (top, Edge::Top),
            (1.0 - top, Edge::Bottom),
        ]
        .into_iter()
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .filter(|(distance, _)| *distance < EDGE_BAND);
        Some(nearest.map_or(DropZone::Center, |(_, edge)| DropZone::Edge(edge)))
    }

    pub fn area(self, rect: Rect) -> Rect {
        let width = (rect.width * HALF).round();
        let height = (rect.height * HALF).round();
        match self {
            DropZone::Center => rect,
            DropZone::Edge(Edge::Left) => Rect::new(rect.x, rect.y, width, rect.height),
            DropZone::Edge(Edge::Right) => {
                Rect::new(rect.right() - width, rect.y, width, rect.height)
            }
            DropZone::Edge(Edge::Top) => Rect::new(rect.x, rect.y, rect.width, height),
            DropZone::Edge(Edge::Bottom) => {
                Rect::new(rect.x, rect.bottom() - height, rect.width, height)
            }
        }
    }
}
