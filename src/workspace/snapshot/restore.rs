use std::collections::BTreeMap;

use super::{InstanceSnapshot, NodeSnapshot, PaneSnapshot, Snapshot, SnapshotError, TabSnapshot};
use crate::session::SessionId;
use crate::workspace::{Instance, InstanceId, Node, Pane, PaneId, Workspace};

type Reserve<'a> = &'a mut dyn FnMut(&TabSnapshot) -> SessionId;

const FALLBACK_RATIO: f32 = 1.0;

impl Workspace {
    pub fn restore(snapshot: &Snapshot, reserve: Reserve) -> Result<Workspace, SnapshotError> {
        let mut restorer = Restorer {
            workspace: Workspace::new(),
            reserve,
        };
        restorer.workspace.sidebar_open = snapshot.sidebar_open;
        for item in &snapshot.instances {
            if let Some(instance) = restorer.instance(item) {
                restorer.workspace.instances.push(instance);
            }
        }
        let mut workspace = restorer.workspace;
        if workspace.instances.is_empty() {
            return Err(SnapshotError::Empty);
        }
        workspace.active = snapshot.active_instance.min(workspace.instances.len() - 1);
        Ok(workspace)
    }
}

struct Restorer<'a> {
    workspace: Workspace,
    reserve: Reserve<'a>,
}

impl Restorer<'_> {
    fn instance(&mut self, item: &InstanceSnapshot) -> Option<Instance> {
        let mut panes = BTreeMap::new();
        let root = self.node(&item.root, &mut panes)?;
        let order = root.panes();
        let active_pane = *order.get(item.active_pane).or(order.first())?;
        self.workspace.next_instance += 1;
        let number = self.workspace.next_instance;
        let name = match item.name.trim() {
            "" => format!("Instance {number}"),
            name => name.to_owned(),
        };
        Some(Instance {
            id: InstanceId(number),
            name,
            root,
            panes,
            active_pane,
        })
    }

    fn node(&mut self, node: &NodeSnapshot, panes: &mut BTreeMap<PaneId, Pane>) -> Option<Node> {
        match node {
            NodeSnapshot::Pane(item) => {
                let pane = self.pane(item)?;
                let id = self.workspace.new_pane_id();
                panes.insert(id, pane);
                Some(Node::Pane(id))
            }
            NodeSnapshot::Split {
                axis,
                ratios,
                pinned,
                children,
            } => {
                let aligned = pinned.len() == children.len();
                let mut kept = Vec::new();
                let mut flexes = Vec::new();
                let mut kept_pins = Vec::new();
                for (index, child) in children.iter().enumerate() {
                    if let Some(node) = self.node(child, panes) {
                        kept.push(node);
                        flexes.push(usable_ratio(ratios.get(index).copied()));
                        kept_pins.push(aligned && pinned[index]);
                    }
                }
                match kept.len() {
                    0 => None,
                    1 => kept.pop(),
                    _ => Some(Node::Split {
                        axis: *axis,
                        children: kept,
                        flexes,
                        pinned: kept_pins,
                    }),
                }
            }
        }
    }

    fn pane(&mut self, item: &PaneSnapshot) -> Option<Pane> {
        if item.tabs.is_empty() {
            return None;
        }
        let tabs: Vec<SessionId> = item.tabs.iter().map(|tab| (self.reserve)(tab)).collect();
        let active = item.active.min(tabs.len() - 1);
        Some(Pane { tabs, active })
    }
}

fn usable_ratio(ratio: Option<f32>) -> f32 {
    ratio
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(FALLBACK_RATIO)
}
