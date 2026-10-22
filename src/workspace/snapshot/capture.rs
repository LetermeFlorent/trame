use std::path::PathBuf;

use super::{
    InstanceSnapshot, NodeSnapshot, PaneSnapshot, SNAPSHOT_VERSION, Snapshot, TabSnapshot,
};
use crate::session::SessionId;
use crate::workspace::{Instance, Node, Pane, Workspace};

type CwdOf<'a> = &'a dyn Fn(SessionId) -> Option<PathBuf>;

impl Workspace {
    pub fn snapshot(&self, font_size: f32, cwd: CwdOf) -> Snapshot {
        Snapshot {
            version: SNAPSHOT_VERSION,
            sidebar_open: self.sidebar_open,
            font_size,
            active_instance: self.active,
            window: None,
            instances: self
                .instances
                .iter()
                .map(|instance| capture_instance(self, instance, cwd))
                .collect(),
        }
    }
}

fn capture_instance(workspace: &Workspace, instance: &Instance, cwd: CwdOf) -> InstanceSnapshot {
    let order = instance.root.panes();
    InstanceSnapshot {
        name: instance.name.clone(),
        color: instance.color,
        active_pane: order
            .iter()
            .position(|id| *id == instance.active_pane)
            .unwrap_or(0),
        root: capture_node(workspace, instance, &instance.root, cwd),
    }
}

fn capture_node(
    workspace: &Workspace,
    instance: &Instance,
    node: &Node,
    cwd: CwdOf,
) -> NodeSnapshot {
    match node {
        Node::Pane(id) => NodeSnapshot::Pane(
            instance
                .panes
                .get(id)
                .map(|pane| capture_pane(workspace, pane, cwd))
                .unwrap_or_default(),
        ),
        Node::Split {
            axis,
            children,
            flexes,
            pinned,
        } => NodeSnapshot::Split {
            axis: *axis,
            ratios: flexes.clone(),
            pinned: pinned.clone(),
            children: children
                .iter()
                .map(|child| capture_node(workspace, instance, child, cwd))
                .collect(),
        },
    }
}

fn capture_pane(workspace: &Workspace, pane: &Pane, cwd: CwdOf) -> PaneSnapshot {
    PaneSnapshot {
        tabs: pane
            .tabs
            .iter()
            .map(|tab| TabSnapshot {
                cwd: cwd(*tab).filter(|path| path.to_str().is_some()),
                title: workspace.tab_name(*tab).map(str::to_owned),
                color: workspace.tab_color(*tab),
            })
            .collect(),
        active: pane.active,
    }
}
