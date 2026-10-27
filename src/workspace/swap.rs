use super::instance::InstanceId;
use super::pane::Pane;
use super::tree::PaneId;
use super::{TabLocation, Workspace};
use crate::session::SessionId;

impl Workspace {
    pub(super) fn swap(
        &mut self,
        tab: SessionId,
        source: TabLocation,
        instance: InstanceId,
        pane: PaneId,
    ) -> bool {
        let Some(at) = self.instance_index(instance) else {
            return false;
        };
        if at == source.instance && pane == source.pane {
            return false;
        }
        let Some(other) = self.instances[at]
            .panes
            .get(&pane)
            .and_then(Pane::active_tab)
        else {
            return false;
        };
        if let Some(origin) = self.instances[source.instance].panes.get_mut(&source.pane) {
            origin.tabs[source.index] = other;
            origin.active = source.index;
        }
        let target = &mut self.instances[at];
        if let Some(destination) = target.panes.get_mut(&pane) {
            let index = destination.active;
            destination.tabs[index] = tab;
        }
        target.focus(pane);
        self.active = at;
        true
    }
}
