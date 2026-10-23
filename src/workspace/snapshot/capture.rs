use super::{
    InstanceSnapshot, NodeSnapshot, PaneSnapshot, SNAPSHOT_VERSION, Snapshot, TabSnapshot,
};
use crate::preferences::Preferences;
use crate::session::SessionId;
use crate::workspace::{Instance, Node, Pane, Workspace};

pub type DescribeTab<'a> = &'a dyn Fn(SessionId) -> TabSnapshot;

impl Workspace {
    pub fn snapshot(&self, preferences: Preferences, describe: DescribeTab) -> Snapshot {
        Snapshot {
            version: SNAPSHOT_VERSION,
            resume: false,
            sidebar_open: self.sidebar_open,
            sidebar_width: self.sidebar_width,
            preferences,
            active_instance: self.active,
            window: None,
            instances: self
                .instances
                .iter()
                .map(|instance| capture_instance(self, instance, describe))
                .collect(),
        }
    }
}

fn capture_instance(
    workspace: &Workspace,
    instance: &Instance,
    describe: DescribeTab,
) -> InstanceSnapshot {
    let order = instance.root.panes();
    InstanceSnapshot {
        name: instance.name.clone(),
        color: instance.color,
        emblem: Some(instance.emblem),
        folder: instance
            .folder
            .clone()
            .filter(|path| path.to_str().is_some()),
        active_pane: order
            .iter()
            .position(|id| *id == instance.active_pane)
            .unwrap_or(0),
        root: capture_node(workspace, instance, &instance.root, describe),
    }
}

fn capture_node(
    workspace: &Workspace,
    instance: &Instance,
    node: &Node,
    describe: DescribeTab,
) -> NodeSnapshot {
    match node {
        Node::Pane(id) => NodeSnapshot::Pane(
            instance
                .panes
                .get(id)
                .map(|pane| capture_pane(workspace, pane, describe))
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
                .map(|child| capture_node(workspace, instance, child, describe))
                .collect(),
        },
    }
}

fn capture_pane(workspace: &Workspace, pane: &Pane, describe: DescribeTab) -> PaneSnapshot {
    PaneSnapshot {
        tabs: pane
            .tabs
            .iter()
            .map(|tab| capture_tab(workspace, *tab, describe))
            .collect(),
        active: pane.active,
    }
}

fn capture_tab(workspace: &Workspace, id: SessionId, describe: DescribeTab) -> TabSnapshot {
    let described = describe(id);
    TabSnapshot {
        cwd: described.cwd.filter(|path| path.to_str().is_some()),
        title: workspace.tab_name(id).map(str::to_owned),
        color: workspace.tab_color(id),
        transcript: described.transcript,
        ssh: workspace.tab_host(id).cloned(),
    }
}
