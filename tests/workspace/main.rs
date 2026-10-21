mod layouts;
mod lifecycle;
mod moving;
mod panes;
mod tree;

use trame::session::SessionId;
use trame::workspace::{Axis, Node, Pane, PaneId, Workspace};

pub fn tab(id: u64) -> SessionId {
    SessionId(id)
}

pub fn tabs(ids: &[u64]) -> Vec<SessionId> {
    ids.iter().copied().map(tab).collect()
}

fn pane_with(ids: &[u64], active: usize) -> Pane {
    let mut pane = Pane::new(tab(ids[0]));
    for id in &ids[1..] {
        pane.insert(tab(*id), None);
    }
    pane.active = active;
    pane
}

pub fn leaf(id: u32) -> Node {
    Node::Pane(PaneId(id))
}

pub fn weighted(axis: Axis, children: Vec<Node>, flexes: Vec<f32>) -> Node {
    Node::Split {
        axis,
        children,
        flexes,
    }
}

pub fn row(children: Vec<Node>) -> Node {
    let flexes = vec![1.0; children.len()];
    weighted(Axis::Row, children, flexes)
}

pub fn column(children: Vec<Node>) -> Node {
    let flexes = vec![1.0; children.len()];
    weighted(Axis::Column, children, flexes)
}

pub fn workspace_with(ids: &[u64]) -> Workspace {
    let mut workspace = Workspace::new();
    workspace.add_instance(None, tab(ids[0]));
    for id in &ids[1..] {
        workspace.add_tab(tab(*id));
    }
    workspace
}

pub fn pane_of(workspace: &Workspace, id: u64) -> PaneId {
    workspace
        .locate(tab(id))
        .map(|location| location.pane)
        .expect("tab is open")
}

pub fn pane_tabs(workspace: &Workspace, pane: PaneId) -> Vec<SessionId> {
    workspace
        .instances
        .iter()
        .find_map(|instance| instance.panes.get(&pane))
        .map(|pane| pane.tabs.clone())
        .unwrap_or_default()
}

pub fn assert_unchanged(workspace: &Workspace, before: &Workspace) {
    assert_eq!(workspace.instances, before.instances);
    assert_eq!(workspace.active, before.active);
}
