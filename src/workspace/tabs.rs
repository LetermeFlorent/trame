use super::instance::Instance;
use super::tree::Edge;
use super::{TabLocation, Workspace};
use crate::session::SessionId;

impl Workspace {
    pub fn add_tab(&mut self, tab: SessionId) {
        match self.active_instance_mut().and_then(Instance::active_mut) {
            Some(pane) => pane.insert(tab, Some(pane.active + 1)),
            None => {
                self.add_instance(None, tab);
            }
        }
    }

    pub fn split_active(&mut self, tab: SessionId, edge: Edge) {
        let new = self.new_pane_id();
        let Some(instance) = self.active_instance_mut() else {
            self.add_instance(None, tab);
            return;
        };
        let target = instance.active_pane;
        instance.split(target, new, tab, edge);
    }

    pub fn locate(&self, tab: SessionId) -> Option<TabLocation> {
        self.instances
            .iter()
            .enumerate()
            .find_map(|(instance, item)| {
                let pane = item.find(tab)?;
                let index = item.panes[&pane].position(tab)?;
                Some(TabLocation {
                    instance,
                    pane,
                    index,
                })
            })
    }

    pub fn activate_tab(&mut self, tab: SessionId) {
        let Some(location) = self.locate(tab) else {
            return;
        };
        self.active = location.instance;
        let instance = &mut self.instances[location.instance];
        instance.focus(location.pane);
        if let Some(pane) = instance.panes.get_mut(&location.pane) {
            pane.activate(tab);
        }
    }

    pub fn replace_tab(&mut self, old: SessionId, new: SessionId) -> bool {
        let Some(location) = self.locate(old) else {
            return false;
        };
        let instance = &mut self.instances[location.instance];
        let Some(pane) = instance.panes.get_mut(&location.pane) else {
            return false;
        };
        pane.tabs[location.index] = new;
        if let Some(mark) = self.marks.remove(&old) {
            self.marks.insert(new, mark);
        }
        true
    }

    pub fn close_tab(&mut self, tab: SessionId) -> bool {
        let Some(location) = self.locate(tab) else {
            return false;
        };
        self.marks.remove(&tab);
        let instance = &mut self.instances[location.instance];
        instance.remove_tab(tab);
        if instance.is_empty() {
            self.remove_instance(location.instance);
        }
        true
    }
}
