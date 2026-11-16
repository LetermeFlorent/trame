use crate::workspace::{Edge, Instance, PaneLayout};

use super::super::{Desk, Effects};

impl Desk {
    pub(super) fn cycle_tab(&mut self, step: isize) -> Effects {
        if let Some(pane) = self
            .workspace
            .active_instance_mut()
            .and_then(Instance::active_mut)
        {
            pane.cycle(step);
        }
        Effects::default()
    }

    pub(super) fn cycle_instance(&mut self, step: isize) -> Effects {
        let count = self.workspace.instances.len() as isize;
        if count > 1 {
            let next = (self.workspace.active as isize + step).rem_euclid(count);
            self.workspace.active = next as usize;
        }
        Effects::default()
    }

    pub(super) fn focus_neighbor(&mut self, edge: Edge, panes: &PaneLayout) -> Effects {
        if let Some(instance) = self.workspace.active_instance_mut()
            && let Some(next) = panes.neighbor(instance.active_pane, edge)
        {
            instance.focus(next);
        }
        Effects::default()
    }
}
