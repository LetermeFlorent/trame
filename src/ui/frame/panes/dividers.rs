use std::collections::HashMap;

use crate::geometry::Rect;
use crate::workspace::{Axis, Divider, Node, PaneId};

pub(super) fn follow(
    divider: &Divider,
    root: &Node,
    shown: &HashMap<PaneId, Rect>,
    targets: &HashMap<PaneId, Rect>,
) -> Divider {
    let Some(parent @ Node::Split { children, .. }) = root.node(&divider.path) else {
        return divider.clone();
    };
    let spans = children
        .get(divider.index)
        .and_then(|before| Some((span(before, shown)?, span(before, targets)?)));
    let (Some((before, before_target)), Some(whole), Some(whole_target)) =
        (spans, span(parent, shown), span(parent, targets))
    else {
        return divider.clone();
    };
    let rect = divider.rect;
    let rect = match divider.axis {
        Axis::Row => Rect::new(
            rect.x + before.right() - before_target.right(),
            rect.y + whole.y - whole_target.y,
            rect.width,
            rect.height + whole.height - whole_target.height,
        ),
        Axis::Column => Rect::new(
            rect.x + whole.x - whole_target.x,
            rect.y + before.bottom() - before_target.bottom(),
            rect.width + whole.width - whole_target.width,
            rect.height,
        ),
    };
    Divider {
        rect,
        ..divider.clone()
    }
}

fn span(node: &Node, rects: &HashMap<PaneId, Rect>) -> Option<Rect> {
    node.panes()
        .iter()
        .filter_map(|id| rects.get(id))
        .copied()
        .reduce(|all, rect| all.union(rect))
}
