use super::instance::InstanceId;
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
        let source_id = self.instances[source.instance].id;
        let source_tabs = self.instances[source.instance].panes[&source.pane]
            .tabs
            .len();
        match target {
            DropTarget::Tabs {
                pane,
                index,
                instance,
            } if pane == source.pane && instance == source_id => {
                let index = index.unwrap_or(source_tabs);
                if let Some(pane) = self.instances[source.instance].panes.get_mut(&pane) {
                    pane.reorder(tab, index);
                }
                true
            }
            DropTarget::Tabs {
                instance,
                pane,
                index,
            } => self.transfer(tab, source, instance, |workspace, at| {
                let target = &mut workspace.instances[at];
                let Some(destination) = target.panes.get_mut(&pane) else {
                    return false;
                };
                destination.insert(tab, index);
                target.focus(pane);
                true
            }),
            DropTarget::Split { pane, .. } if pane == source.pane && source_tabs == 1 => false,
            DropTarget::Split {
                instance,
                pane,
                edge,
            } => {
                let new = self.new_pane_id();
                self.transfer(tab, source, instance, |workspace, at| {
                    workspace.instances[at].split(pane, new, tab, edge)
                })
            }
            DropTarget::Instance(instance) if instance == source_id => false,
            DropTarget::Instance(instance) => {
                self.transfer(tab, source, instance, |workspace, at| {
                    match workspace.instances[at].active_mut() {
                        Some(pane) => {
                            pane.insert(tab, None);
                            true
                        }
                        None => false,
                    }
                })
            }
            DropTarget::NewInstance if self.instances[source.instance].tab_count() == 1 => false,
            DropTarget::NewInstance => {
                self.detach(tab, source);
                self.add_instance(None, tab);
                true
            }
        }
    }

    fn transfer(
        &mut self,
        tab: SessionId,
        source: TabLocation,
        target: InstanceId,
        place: impl FnOnce(&mut Workspace, usize) -> bool,
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
        if !place(self, at) {
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
