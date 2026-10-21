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
    },
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

    pub fn split(&mut self, target: PaneId, new: PaneId, edge: Edge) -> bool {
        match self {
            Node::Pane(id) if *id == target => {
                let pair = if edge.is_leading() {
                    vec![Node::Pane(new), Node::Pane(target)]
                } else {
                    vec![Node::Pane(target), Node::Pane(new)]
                };
                *self = Node::Split {
                    axis: edge.axis(),
                    children: pair,
                    flexes: vec![1.0, 1.0],
                };
                true
            }
            Node::Pane(_) => false,
            Node::Split {
                axis,
                children,
                flexes,
            } => {
                let direct = children
                    .iter()
                    .position(|child| *child == Node::Pane(target));
                if let Some(index) = direct.filter(|_| *axis == edge.axis()) {
                    let half = flexes[index] / 2.0;
                    flexes[index] = half;
                    let at = if edge.is_leading() { index } else { index + 1 };
                    children.insert(at, Node::Pane(new));
                    flexes.insert(at, half);
                    return true;
                }
                children.iter_mut().any(|child| child.split(target, new, edge))
            }
        }
    }

    pub fn remove(&mut self, pane: PaneId) -> bool {
        let Node::Split {
            children, flexes, ..
        } = self
        else {
            return false;
        };
        let removed = match children.iter().position(|child| *child == Node::Pane(pane)) {
            Some(index) => {
                children.remove(index);
                flexes.remove(index);
                true
            }
            None => children.iter_mut().any(|child| child.remove(pane)),
        };
        if removed {
            self.normalize();
        }
        removed
    }

    fn normalize(&mut self) {
        let Node::Split {
            axis,
            children,
            flexes,
        } = self
        else {
            return;
        };
        if children.len() == 1 {
            *self = children.remove(0);
            return;
        }
        let mut merged_children = Vec::with_capacity(children.len());
        let mut merged_flexes = Vec::with_capacity(flexes.len());
        for (child, flex) in children.drain(..).zip(flexes.drain(..)) {
            match child {
                Node::Split {
                    axis: inner,
                    children: inner_children,
                    flexes: inner_flexes,
                } if inner == *axis => {
                    let total: f32 = inner_flexes.iter().sum();
                    merged_flexes.extend(inner_flexes.iter().map(|value| value / total * flex));
                    merged_children.extend(inner_children);
                }
                other => {
                    merged_children.push(other);
                    merged_flexes.push(flex);
                }
            }
        }
        *children = merged_children;
        *flexes = merged_flexes;
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
