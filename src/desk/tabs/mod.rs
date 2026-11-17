mod exits;
mod folder;
mod ssh;
mod start;

use std::path::PathBuf;

use crate::host::quit::Quit;
use crate::preferences::SshHost;
use crate::session::SessionId;
use crate::workspace::{Edge, InstanceId, PaneId};

use super::{Desk, Effect, Effects, Grids, Outcome};
pub use folder::FolderTarget;
pub(super) use start::Starting;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Tab,
    Split(Edge),
    Instance,
}

impl Desk {
    pub fn open(&mut self, placement: Placement) -> Effects {
        self.open_at(placement, None)
    }

    pub fn open_at(&mut self, placement: Placement, folder: Option<PathBuf>) -> Effects {
        let cwd = folder.clone().or_else(|| self.start_folder(placement));
        let opened = self.open_in(placement, cwd.clone(), None);
        let mut effects = opened.effects;
        if placement == Placement::Instance {
            let instance = self.workspace.active_instance().map(|instance| instance.id);
            if let (Some(id), Some(folder)) = (instance, folder) {
                self.workspace.set_instance_folder(id, Some(folder));
            }
            for _ in 1..self.preferences.instance_tabs {
                effects.append(self.open_in(Placement::Tab, cwd.clone(), None).effects);
            }
            self.workspace.activate_tab(opened.value);
            effects.push(Effect::Changed);
        }
        effects
    }

    fn open_in(
        &mut self,
        placement: Placement,
        cwd: Option<PathBuf>,
        host: Option<SshHost>,
    ) -> Outcome<SessionId> {
        let id = self.sessions.reserve();
        match placement {
            Placement::Tab => self.workspace.add_tab(id),
            Placement::Split(edge) => self.workspace.split_active(id, edge),
            Placement::Instance => {
                self.workspace.add_instance(None, id);
            }
        }
        if let Some(host) = host {
            self.workspace.rename_tab(id, &host.label());
            self.workspace.set_tab_host(id, Some(host));
        }
        Outcome {
            value: id,
            effects: self.queue_starts(vec![(id, cwd)]),
        }
    }

    pub fn close_tab(&mut self, exiting: bool, id: SessionId) -> Effects {
        let mut effects = Effects::default();
        if self.workspace.is_only_tab(id) && !exiting && !self.quitting() {
            effects.append(self.replace_last_tab());
        }
        if self.workspace.close_tab(id) {
            self.sessions.remove(id);
            effects.append(self.after_close());
        }
        effects
    }

    pub fn close_instance(&mut self, instance: InstanceId) -> Effects {
        let Some(index) = self.workspace.instance_index(instance) else {
            return Effects::default();
        };
        for id in self.workspace.remove_instance(index) {
            self.sessions.remove(id);
        }
        self.after_close()
    }

    pub fn after_close(&mut self) -> Effects {
        if self.workspace.is_empty() {
            self.quit_with(Quit::Close);
            return Effects::default();
        }
        Effects::from([Effect::Animate, Effect::Changed])
    }

    pub fn focus_pane(&mut self, pane: PaneId) {
        if let Some(instance) = self.workspace.active_instance_mut() {
            instance.focus(pane);
        }
    }

    pub fn activate_instance(&mut self, instance: InstanceId) {
        if let Some(index) = self.workspace.instance_index(instance) {
            self.workspace.active = index;
        }
    }

    pub fn resize_shells(&mut self, grids: &Grids) {
        for (id, size) in grids.sizes() {
            self.sessions.resize(id, size, grids.cell_pixels());
        }
    }
}
