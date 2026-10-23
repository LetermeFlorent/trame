mod folder;
mod ssh;

use std::io;
use std::path::Path;

use winit::event_loop::ActiveEventLoop;

use trame::session::SessionId;
use trame::workspace::{Edge, InstanceId, PaneId};

use super::Running;
use crate::app::sessions::ShellStart;
pub use folder::FolderTarget;

pub(super) const FALLBACK_GRID: (usize, usize) = (24, 80);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Placement {
    Tab,
    Split(Edge),
    Instance,
}

impl Running {
    pub(super) fn open(&mut self, placement: Placement) -> io::Result<SessionId> {
        let cwd = self.start_folder(placement);
        let id = self.sessions.reserve();
        match placement {
            Placement::Tab => self.workspace.add_tab(id),
            Placement::Split(edge) => self.workspace.split_active(id, edge),
            Placement::Instance => {
                self.workspace.add_instance(None, id);
            }
        }
        self.relayout();
        let spawned = self.start_shell(id, cwd.as_deref(), &[]);
        self.changed();
        spawned.map(|()| id)
    }

    pub(super) fn start_shell(
        &mut self,
        id: SessionId,
        cwd: Option<&Path>,
        transcript: &[u8],
    ) -> io::Result<()> {
        let start = ShellStart {
            cwd,
            size: self.grid_size_of(id).unwrap_or(FALLBACK_GRID),
            colors: self.theme.reported_colors(),
            transcript,
        };
        let spawned = self.sessions.spawn(id, &start);
        match spawned {
            Ok(()) => self.dial(id),
            Err(_) => {
                self.workspace.close_tab(id);
            }
        }
        spawned
    }

    pub(super) fn open_or_report(&mut self, event_loop: &ActiveEventLoop, placement: Placement) {
        if let Err(error) = self.open(placement) {
            eprintln!("trame: the shell could not start: {error}");
            if self.workspace.is_empty() {
                event_loop.exit();
            }
        }
    }

    pub(super) fn close_tab(&mut self, event_loop: &ActiveEventLoop, id: SessionId) {
        if self.workspace.close_tab(id) {
            self.sessions.remove(id);
            self.after_close(event_loop);
        }
    }

    pub(super) fn close_instance(&mut self, event_loop: &ActiveEventLoop, instance: InstanceId) {
        let Some(index) = self.workspace.instance_index(instance) else {
            return;
        };
        for id in self.workspace.remove_instance(index) {
            self.sessions.remove(id);
        }
        self.after_close(event_loop);
    }

    pub(super) fn focus_pane(&mut self, pane: PaneId) {
        if let Some(instance) = self.workspace.active_instance_mut() {
            instance.focus(pane);
        }
    }

    pub(super) fn activate_instance(&mut self, instance: InstanceId) {
        if let Some(index) = self.workspace.instance_index(instance) {
            self.workspace.active = index;
        }
    }

    fn after_close(&mut self, event_loop: &ActiveEventLoop) {
        if self.workspace.is_empty() {
            event_loop.exit();
            return;
        }
        self.motion.animate_next();
        self.changed();
    }

    pub(super) fn grid_size_of(&self, id: SessionId) -> Option<(usize, usize)> {
        let cell = self.text.cell_metrics();
        self.frame
            .panes
            .iter()
            .find(|pane| pane.tabs.contains(&id))
            .map(|pane| pane.grid_size(&cell))
    }
}
