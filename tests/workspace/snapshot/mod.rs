mod invalid;
mod marks;
mod nesting;
mod preferences;
mod round_trip;
mod sanitize;

use std::collections::HashMap;
use std::path::PathBuf;

use trame::preferences::Preferences;
use trame::session::SessionId;
use trame::workspace::{
    Instance, InstanceSnapshot, Node, NodeSnapshot, PaneSnapshot, SNAPSHOT_VERSION, Snapshot,
    TabSnapshot, Workspace,
};

pub const FIRST_RESTORED_ID: u64 = 100;

pub fn snapshot_of(workspace: &Workspace, cwds: &[(u64, &str)]) -> Snapshot {
    let known: HashMap<SessionId, PathBuf> = cwds
        .iter()
        .map(|(id, path)| (SessionId(*id), PathBuf::from(path)))
        .collect();
    workspace.snapshot(Preferences::default(), &|id| TabSnapshot {
        cwd: known.get(&id).cloned(),
        ..TabSnapshot::default()
    })
}

pub fn restore(snapshot: &Snapshot) -> (Workspace, Vec<TabSnapshot>) {
    let mut seen = Vec::new();
    let mut next = FIRST_RESTORED_ID;
    let workspace = Workspace::restore(snapshot, &mut |tab| {
        next += 1;
        seen.push(tab.clone());
        SessionId(next)
    })
    .expect("the snapshot restores");
    (workspace, seen)
}

pub fn shape(instance: &Instance) -> String {
    describe(instance, &instance.root)
}

fn describe(instance: &Instance, node: &Node) -> String {
    match node {
        Node::Pane(id) => format!("[{}]", instance.panes[id].tabs.len()),
        Node::Split {
            axis,
            children,
            flexes,
            ..
        } => {
            let parts: Vec<String> = children
                .iter()
                .zip(flexes)
                .map(|(child, flex)| format!("{flex}:{}", describe(instance, child)))
                .collect();
            format!("{axis:?}({})", parts.join(","))
        }
    }
}

pub fn pane(tabs: usize) -> NodeSnapshot {
    NodeSnapshot::Pane(PaneSnapshot {
        tabs: vec![TabSnapshot::default(); tabs],
        active: 0,
    })
}

pub fn single(root: NodeSnapshot) -> Snapshot {
    Snapshot {
        version: SNAPSHOT_VERSION,
        sidebar_open: true,
        sidebar_width: None,
        preferences: Preferences::default(),
        active_instance: 0,
        window: None,
        instances: vec![instance_of(root)],
    }
}

pub fn instance_of(root: NodeSnapshot) -> InstanceSnapshot {
    InstanceSnapshot {
        name: "work".to_owned(),
        color: None,
        emblem: None,
        active_pane: 0,
        root,
    }
}
