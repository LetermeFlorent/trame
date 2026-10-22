use std::collections::HashMap;

use crate::geometry::Rect;
use crate::ui::motion::{Motion, MotionKey};
use crate::workspace::{PaneId, PaneLayout};

pub(super) fn entries(layout: &PaneLayout, motion: &Motion) -> HashMap<PaneId, Rect> {
    layout
        .panes
        .iter()
        .filter(|(id, _)| !motion.knows(MotionKey::Pane(*id)))
        .filter_map(|(id, target)| Some((*id, entry(layout, motion, *target)?)))
        .collect()
}

fn entry(layout: &PaneLayout, motion: &Motion, target: Rect) -> Option<Rect> {
    let (_, survivor) = layout
        .panes
        .iter()
        .filter_map(|(id, rect)| {
            let previous = motion.previous(MotionKey::Pane(*id))?;
            let overlap = previous.intersect(target);
            let area = overlap.width * overlap.height;
            (area > 0.0).then_some((area, *rect))
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))?;
    edge_entry(target, survivor)
}

fn edge_entry(target: Rect, survivor: Rect) -> Option<Rect> {
    if target.x >= survivor.right() {
        return Some(Rect::new(target.right(), target.y, 0.0, target.height));
    }
    if target.right() <= survivor.x {
        return Some(Rect::new(target.x, target.y, 0.0, target.height));
    }
    if target.y >= survivor.bottom() {
        return Some(Rect::new(target.x, target.bottom(), target.width, 0.0));
    }
    if target.bottom() <= survivor.y {
        return Some(Rect::new(target.x, target.y, target.width, 0.0));
    }
    None
}
