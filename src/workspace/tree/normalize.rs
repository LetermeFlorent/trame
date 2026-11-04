use super::{Axis, Node, sync_pins};

impl Node {
    pub(super) fn normalize(&mut self) {
        let Node::Split {
            axis,
            children,
            flexes,
            pinned,
        } = self
        else {
            return;
        };
        if children.len() == 1 {
            *self = children.remove(0);
            return;
        }
        sync_pins(pinned, flexes.len());
        let mut merged = Merged {
            axis: *axis,
            children: Vec::new(),
            flexes: Vec::new(),
            pinned: Vec::new(),
        };
        let slots = children
            .drain(..)
            .zip(flexes.drain(..))
            .zip(pinned.drain(..));
        for ((child, flex), pin) in slots {
            merged.absorb(child, flex, pin);
        }
        *children = merged.children;
        *flexes = merged.flexes;
        *pinned = merged.pinned;
    }
}

struct Merged {
    axis: Axis,
    children: Vec<Node>,
    flexes: Vec<f32>,
    pinned: Vec<bool>,
}

impl Merged {
    fn absorb(&mut self, child: Node, flex: f32, pin: bool) {
        match child {
            Node::Split {
                axis,
                children,
                flexes,
                mut pinned,
            } if axis == self.axis => {
                sync_pins(&mut pinned, flexes.len());
                let total: f32 = flexes.iter().sum();
                let scaled = flexes.iter().map(|value| value / total * flex);
                self.flexes.extend(scaled);
                self.pinned.extend(pinned.iter().map(|inner| *inner || pin));
                self.children.extend(children);
            }
            other => {
                self.children.push(other);
                self.flexes.push(flex);
                self.pinned.push(pin);
            }
        }
    }
}
