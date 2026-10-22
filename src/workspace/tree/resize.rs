use super::layout::Divider;
use super::{Node, sync_pins};

pub const MIN_PANE: f32 = 80.0;

pub fn resize_split(root: &mut Node, divider: &Divider, delta: f32) {
    let Some(Node::Split { flexes, pinned, .. }) = root.node_mut(&divider.path) else {
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
    if first == flexes[index] {
        return;
    }
    flexes[index] = first;
    flexes[index + 1] = pair - first;
    sync_pins(pinned, flexes.len());
    pinned[index] = true;
    pinned[index + 1] = true;
}

pub fn resized(origin: &Node, divider: &Divider, delta: f32) -> Node {
    let mut root = origin.clone();
    resize_split(&mut root, divider, delta);
    root
}
