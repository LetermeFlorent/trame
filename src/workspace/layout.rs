use super::tree::{Axis, Edge, Node, PaneId, split_extent};
use crate::geometry::{Point, Rect};

pub const MIN_PANE: f32 = 80.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Divider {
    pub path: Vec<usize>,
    pub index: usize,
    pub axis: Axis,
    pub rect: Rect,
    pub extent: f32,
}

impl Divider {
    pub fn delta(&self, from: Point, to: Point) -> f32 {
        match self.axis {
            Axis::Row => to.x - from.x,
            Axis::Column => to.y - from.y,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaneLayout {
    pub panes: Vec<(PaneId, Rect)>,
    pub dividers: Vec<Divider>,
}

impl PaneLayout {
    pub fn compute(root: &Node, rect: Rect, gap: f32) -> Self {
        let mut layout = Self::default();
        layout.place(root, rect, gap, &mut Vec::new());
        layout
    }

    fn place(&mut self, node: &Node, rect: Rect, gap: f32, path: &mut Vec<usize>) {
        let (axis, children, flexes) = match node {
            Node::Pane(pane) => {
                self.panes.push((*pane, rect.snapped()));
                return;
            }
            Node::Split {
                axis,
                children,
                flexes,
            } => (*axis, children, flexes),
        };
        let total: f32 = flexes.iter().sum::<f32>().max(f32::EPSILON);
        let available = split_extent(rect, axis) - gap * (children.len() - 1) as f32;
        let mut offset = 0.0;
        let mut consumed = 0.0;
        for (index, child) in children.iter().enumerate() {
            consumed += flexes[index];
            let end = (available * consumed / total).round();
            let size = end - offset;
            let start = offset + gap * index as f32;
            let child_rect = match axis {
                Axis::Row => Rect::new(rect.x + start, rect.y, size, rect.height),
                Axis::Column => Rect::new(rect.x, rect.y + start, rect.width, size),
            };
            path.push(index);
            self.place(child, child_rect, gap, path);
            path.pop();
            if index + 1 < children.len() {
                let divider = match axis {
                    Axis::Row => Rect::new(child_rect.right(), rect.y, gap, rect.height),
                    Axis::Column => Rect::new(rect.x, child_rect.bottom(), rect.width, gap),
                };
                self.dividers.push(Divider {
                    path: path.clone(),
                    index,
                    axis,
                    rect: divider,
                    extent: available,
                });
            }
            offset = end;
        }
    }

    pub fn rect_of(&self, pane: PaneId) -> Option<Rect> {
        self.panes
            .iter()
            .find(|(id, _)| *id == pane)
            .map(|(_, rect)| *rect)
    }

    pub fn pane_at(&self, point: Point) -> Option<(PaneId, Rect)> {
        self.panes
            .iter()
            .copied()
            .find(|(_, rect)| rect.contains(point))
    }

    pub fn neighbor(&self, from: PaneId, edge: Edge) -> Option<PaneId> {
        let origin = self.rect_of(from)?;
        let center = origin.center();
        self.panes
            .iter()
            .filter(|(id, _)| *id != from)
            .filter_map(|(id, rect)| {
                let (distance, overlap) = match edge {
                    Edge::Left => (origin.x - rect.right(), overlap_y(origin, *rect)),
                    Edge::Right => (rect.x - origin.right(), overlap_y(origin, *rect)),
                    Edge::Top => (origin.y - rect.bottom(), overlap_x(origin, *rect)),
                    Edge::Bottom => (rect.y - origin.bottom(), overlap_x(origin, *rect)),
                };
                (distance >= -1.0 && overlap > 0.0).then(|| {
                    let other = rect.center();
                    let drift = (other.x - center.x).abs() + (other.y - center.y).abs();
                    (*id, distance, drift)
                })
            })
            .min_by(|a, b| {
                (a.1, a.2)
                    .partial_cmp(&(b.1, b.2))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, _, _)| id)
    }
}

pub fn resize_split(root: &mut Node, divider: &Divider, delta: f32) {
    let Some(Node::Split { flexes, .. }) = root.node_mut(&divider.path) else {
        return;
    };
    let index = divider.index;
    if index + 1 >= flexes.len() || divider.extent <= 0.0 {
        return;
    }
    let total: f32 = flexes.iter().sum();
    let per_pixel = total / divider.extent;
    let pair = flexes[index] + flexes[index + 1];
    let minimum = (MIN_PANE * per_pixel).min(pair / 2.0);
    let first = (flexes[index] + delta * per_pixel).clamp(minimum, pair - minimum);
    flexes[index] = first;
    flexes[index + 1] = pair - first;
}

pub fn resized(origin: &Node, divider: &Divider, delta: f32) -> Node {
    let mut root = origin.clone();
    resize_split(&mut root, divider, delta);
    root
}

fn overlap_x(a: Rect, b: Rect) -> f32 {
    a.right().min(b.right()) - a.x.max(b.x)
}

fn overlap_y(a: Rect, b: Rect) -> f32 {
    a.bottom().min(b.bottom()) - a.y.max(b.y)
}
