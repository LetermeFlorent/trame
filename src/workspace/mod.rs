mod instance;
mod marks;
mod moves;
mod pane;
mod snapshot;
mod tabs;
mod tree;

pub use instance::{Instance, InstanceId};
pub use marks::{TabMark, clean_name};
pub use moves::DropTarget;
pub use pane::Pane;
pub use snapshot::{
    InstanceSnapshot, NodeSnapshot, PaneSnapshot, SNAPSHOT_VERSION, Snapshot, SnapshotError,
    TabSnapshot, WindowSnapshot,
};
pub use tree::{Axis, Divider, Edge, MIN_PANE, Node, PaneId, PaneLayout, resize_split, resized};

use std::collections::HashMap;

use crate::session::SessionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabLocation {
    pub instance: usize,
    pub pane: PaneId,
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Workspace {
    pub instances: Vec<Instance>,
    pub active: usize,
    pub sidebar_open: bool,
    pub sidebar_width: Option<f32>,
    marks: HashMap<SessionId, TabMark>,
    next_pane: u32,
    next_instance: u32,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
            active: 0,
            sidebar_open: true,
            sidebar_width: None,
            marks: HashMap::new(),
            next_pane: 0,
            next_instance: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn active_instance(&self) -> Option<&Instance> {
        self.instances.get(self.active)
    }

    pub fn active_instance_mut(&mut self) -> Option<&mut Instance> {
        self.instances.get_mut(self.active)
    }

    pub fn active_tab(&self) -> Option<SessionId> {
        self.active_instance().and_then(Instance::active_tab)
    }

    pub fn tabs(&self) -> Vec<SessionId> {
        self.instances.iter().flat_map(Instance::tabs).collect()
    }

    pub fn add_instance(&mut self, name: Option<String>, tab: SessionId) -> InstanceId {
        self.next_instance += 1;
        let id = InstanceId(self.next_instance);
        let name = name.unwrap_or_else(|| Instance::automatic_name(id));
        let pane = self.new_pane_id();
        self.instances.push(Instance::new(id, name, pane, tab));
        self.active = self.instances.len() - 1;
        id
    }

    pub fn remove_instance(&mut self, index: usize) -> Vec<SessionId> {
        if index >= self.instances.len() {
            return Vec::new();
        }
        let removed = self.instances.remove(index);
        self.forget_marks(&removed.tabs());
        if self.active > index || self.active >= self.instances.len() {
            self.active = self.active.saturating_sub(1);
        }
        removed.tabs()
    }

    pub fn instance_index(&self, id: InstanceId) -> Option<usize> {
        self.instances.iter().position(|instance| instance.id == id)
    }

    fn new_pane_id(&mut self) -> PaneId {
        self.next_pane += 1;
        PaneId(self.next_pane)
    }
}
