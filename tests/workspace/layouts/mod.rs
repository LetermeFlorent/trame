mod divider;
mod layout;
mod neighbor;
mod resize;

use trame::geometry::Rect;
use trame::workspace::{Divider, Node, PaneLayout};

const AREA: Rect = Rect::new(0.0, 0.0, 200.0, 100.0);

fn sizes(root: &Node, area: Rect, gap: f32, measure: fn(&Rect) -> f32) -> Vec<f32> {
    PaneLayout::compute(root, area, gap)
        .panes
        .iter()
        .map(|(_, rect)| measure(rect))
        .collect()
}

fn widths(root: &Node, area: Rect, gap: f32) -> Vec<f32> {
    sizes(root, area, gap, |rect| rect.width)
}

fn dividers(root: &Node, area: Rect, gap: f32) -> Vec<Divider> {
    PaneLayout::compute(root, area, gap).dividers
}
