use super::instance::{Instance, InstanceId};
use super::tree::{Edge, PaneId};
use super::{TabLocation, Workspace};
use crate::session::SessionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropTarget {
    Tabs {
        instance: InstanceId,
        pane: PaneId,
        index: Option<usize>,
    },
    Split {
        instance: InstanceId,
        pane: PaneId,
        edge: Edge,
    },
    Instance(InstanceId),
    NewInstance,
}

impl Workspace {
    pub fn move_tab(&mut self, tab: SessionId, target: DropTarget) -> bool {
        let Some(source) = self.locate(tab) else {
            return false;
        };
        let origin = &self.instances[source.instance];
        let source_id = origin.id;
        let alone_in_pane = origin.panes[&source.pane].tabs.len() == 1;
        let alone_in_instance = origin.tab_count() == 1;
        match target {
            DropTarget::Tabs {
                instance,
                pane,
                index,
            } if pane == source.pane && instance == source_id => self.reorder(tab, source, index),
            DropTarget::Tabs {
                instance,
                pane,
                index,
            } => self.transfer(tab, source, instance, |into| into.insert(pane, tab, index)),
            DropTarget::Split { pane, .. } if pane == source.pane && alone_in_pane => false,
            DropTarget::Split {
                instance,
                pane,
                edge,
            } => {
                let new = self.new_pane_id();
                self.transfer(tab, source, instance, |into| {
                    into.split(pane, new, tab, edge)
                })
            }
            DropTarget::Instance(instance) if instance == source_id => false,
            DropTarget::Instance(instance) => {
                self.transfer(tab, source, instance, |into| into.insert_active(tab))
            }
            DropTarget::NewInstance if alone_in_instance => false,
            DropTarget::NewInstance => {
                self.detach(tab, source);
                self.add_instance(None, tab);
                true
            }
        }
    }

    fn reorder(&mut self, tab: SessionId, source: TabLocation, index: Option<usize>) -> bool {
        if let Some(pane) = self.instances[source.instance].panes.get_mut(&source.pane) {
            let last = pane.tabs.len();
            pane.reorder(tab, index.unwrap_or(last));
        }
        true
    }

    fn transfer(
        &mut self,
        tab: SessionId,
        source: TabLocation,
        target: InstanceId,
        place: impl FnOnce(&mut Instance) -> bool,
    ) -> bool {
        if self.instance_index(target).is_none() {
            return false;
        }
        let snapshot = self.clone();
        self.detach(tab, source);
        let Some(at) = self.instance_index(target) else {
            *self = snapshot;
            return false;
        };
        if !place(&mut self.instances[at]) {
            *self = snapshot;
            return false;
        }
        self.active = at;
        true
    }

    fn detach(&mut self, tab: SessionId, source: TabLocation) {
        let instance = &mut self.instances[source.instance];
        instance.remove_tab(tab);
        if instance.is_empty() {
            self.remove_instance(source.instance);
        }
    }
}
