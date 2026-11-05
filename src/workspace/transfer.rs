use std::collections::{BTreeMap, HashMap};
use std::mem;

use super::{Instance, InstanceId, TabMark, Workspace};
use crate::session::SessionId;

pub struct MovingInstance {
    pub instance: Instance,
    marks: Vec<(SessionId, TabMark)>,
}

impl Workspace {
    pub fn take_instance(&mut self, id: InstanceId) -> Option<MovingInstance> {
        let index = self.instance_index(id)?;
        let instance = self.instances.remove(index);
        let marks = instance
            .tabs()
            .into_iter()
            .filter_map(|tab| Some((tab, self.marks.remove(&tab)?)))
            .collect();
        if self.active > index || self.active >= self.instances.len() {
            self.active = self.active.saturating_sub(1);
        }
        self.renumber_after_close();
        Some(MovingInstance { instance, marks })
    }

    pub fn adopt_instance(
        &mut self,
        moving: MovingInstance,
        renamed: &HashMap<SessionId, SessionId>,
        index: usize,
    ) {
        let MovingInstance {
            mut instance,
            marks,
        } = moving;
        let tab_of = |tab: SessionId| renamed.get(&tab).copied().unwrap_or(tab);
        let mut pane_of = HashMap::new();
        let mut panes = BTreeMap::new();
        for (old, mut pane) in mem::take(&mut instance.panes) {
            let new = self.new_pane_id();
            pane_of.insert(old, new);
            pane.tabs.iter_mut().for_each(|tab| *tab = tab_of(*tab));
            panes.insert(new, pane);
        }
        instance.panes = panes;
        let pane = |old| pane_of.get(&old).copied().unwrap_or(old);
        instance.root.rename_panes(&pane);
        instance.active_pane = pane(instance.active_pane);
        self.next_instance += 1;
        instance.id = InstanceId(self.next_instance);
        instance.number = self.next_number();
        for (tab, mark) in marks {
            self.mark_tab(tab_of(tab), mark);
        }
        let index = index.min(self.instances.len());
        self.instances.insert(index, instance);
        self.active = index;
        self.renumber_after_move();
    }
}
