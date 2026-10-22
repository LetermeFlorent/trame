use std::collections::BTreeMap;

use super::{InstanceSnapshot, NodeSnapshot, PaneSnapshot, Snapshot, SnapshotError, TabSnapshot};
use crate::session::SessionId;
use crate::theme::Emblem;
use crate::workspace::{Instance, InstanceId, Node, Pane, PaneId, TabMark, Workspace, clean_name};

type Reserve<'a> = &'a mut dyn FnMut(&TabSnapshot) -> SessionId;

const FALLBACK_RATIO: f32 = 1.0;

impl Workspace {
    pub fn restore(snapshot: &Snapshot, reserve: Reserve) -> Result<Workspace, SnapshotError> {
        let mut restorer = Restorer {
            workspace: Workspace::new(),
            reserve,
        };
        restorer.workspace.sidebar_open = snapshot.sidebar_open;
        restorer.workspace.sidebar_width = snapshot
            .sidebar_width
            .filter(|width| width.is_finite() && *width > 0.0);
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
        let id = InstanceId(number);
        let name = clean_name(&item.name).unwrap_or_else(|| Instance::automatic_name(id));
        let emblem = item
            .emblem
            .unwrap_or_else(|| Emblem::random(&self.workspace.emblems()));
        Some(Instance {
            id,
            name,
            color: item.color,
            emblem,
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
        let mut tabs = Vec::with_capacity(item.tabs.len());
        for tab in &item.tabs {
            let id = (self.reserve)(tab);
            let mark = TabMark {
                name: tab.title.as_deref().and_then(clean_name),
                color: tab.color,
            };
            self.workspace.restore_mark(id, mark);
            tabs.push(id);
        }
        let active = item.active.min(tabs.len() - 1);
        Some(Pane { tabs, active })
    }
}

fn usable_ratio(ratio: Option<f32>) -> f32 {
    ratio
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(FALLBACK_RATIO)
}
