use super::{Edge, Node, PaneId, sync_pins};

const MIN_SHARE_RATIO: f32 = 0.05;
const HALF: f32 = 2.0;

impl Node {
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
                    pinned: vec![false, false],
                };
                true
            }
            Node::Pane(_) => false,
            Node::Split {
                axis,
                children,
                flexes,
                pinned,
            } => {
                let direct = children
                    .iter()
                    .position(|child| *child == Node::Pane(target));
                if let Some(index) = direct.filter(|_| *axis == edge.axis()) {
                    sync_pins(pinned, flexes.len());
                    let at = if edge.is_leading() { index } else { index + 1 };
                    share(flexes, pinned, index, at);
                    children.insert(at, Node::Pane(new));
                    return true;
                }
                children
                    .iter_mut()
                    .any(|child| child.split(target, new, edge))
            }
        }
    }
}

fn share(flexes: &mut Vec<f32>, pinned: &mut Vec<bool>, target: usize, at: usize) {
    pinned[target] = false;
    let total: f32 = flexes.iter().sum();
    let pinned_total: f32 = flexes
        .iter()
        .zip(pinned.iter())
        .filter(|(_, pin)| **pin)
        .map(|(flex, _)| flex)
        .sum();
    let members = pinned.iter().filter(|pin| !**pin).count() + 1;
    let each = (total - pinned_total) / members as f32;
    if each >= total * MIN_SHARE_RATIO {
        for (flex, pin) in flexes.iter_mut().zip(pinned.iter()) {
            if !pin {
                *flex = each;
            }
        }
        flexes.insert(at, each);
    } else {
        let half = flexes[target] / HALF;
        flexes[target] = half;
        flexes.insert(at, half);
    }
    pinned.insert(at, false);
}
