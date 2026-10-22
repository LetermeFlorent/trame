use serde::{Deserialize, Serialize};

use crate::geometry::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PaneId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl Edge {
    pub fn axis(self) -> Axis {
        match self {
            Edge::Left | Edge::Right => Axis::Row,
            Edge::Top | Edge::Bottom => Axis::Column,
        }
    }

    pub fn is_leading(self) -> bool {
        matches!(self, Edge::Left | Edge::Top)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Pane(PaneId),
    Split {
        axis: Axis,
        children: Vec<Node>,
        flexes: Vec<f32>,
        #[serde(default)]
        pinned: Vec<bool>,
    },
}

pub(super) fn sync_pins(pinned: &mut Vec<bool>, len: usize) {
    if pinned.len() == len {
        return;
    }
    pinned.clear();
    pinned.resize(len, false);
}

impl Node {
    pub fn panes(&self) -> Vec<PaneId> {
        let mut panes = Vec::new();
        self.collect(&mut panes);
        panes
    }

    fn collect(&self, panes: &mut Vec<PaneId>) {
        match self {
            Node::Pane(pane) => panes.push(*pane),
            Node::Split { children, .. } => children.iter().for_each(|child| child.collect(panes)),
        }
    }

    pub fn contains(&self, pane: PaneId) -> bool {
        match self {
            Node::Pane(id) => *id == pane,
            Node::Split { children, .. } => children.iter().any(|child| child.contains(pane)),
        }
    }

    pub fn remove(&mut self, pane: PaneId) -> bool {
        let Node::Split {
            children,
            flexes,
            pinned,
            ..
        } = self
        else {
            return false;
        };
        let removed = match children.iter().position(|child| *child == Node::Pane(pane)) {
            Some(index) => {
                sync_pins(pinned, flexes.len());
                children.remove(index);
                flexes.remove(index);
                pinned.remove(index);
                true
            }
            None => children.iter_mut().any(|child| child.remove(pane)),
        };
        if removed {
            self.normalize();
        }
        removed
    }

    pub fn node_mut(&mut self, path: &[usize]) -> Option<&mut Node> {
        let Some((first, rest)) = path.split_first() else {
            return Some(self);
        };
        match self {
            Node::Split { children, .. } => children.get_mut(*first)?.node_mut(rest),
            Node::Pane(_) => None,
        }
    }
}

pub fn split_extent(rect: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Row => rect.width,
        Axis::Column => rect.height,
    }
}
