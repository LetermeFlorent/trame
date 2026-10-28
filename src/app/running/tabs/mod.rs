mod closing;
mod confirm;
mod deferred;
mod folder;
mod restarts;
mod ssh;
#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use winit::event_loop::ActiveEventLoop;

use trame::log_error;
use trame::preferences::SshHost;
use trame::session::SessionId;
use trame::workspace::{Edge, InstanceId, PaneId};

use super::Running;
use crate::app::sessions::ShellStart;
pub(super) use closing::Closing;
pub use folder::FolderTarget;
pub(super) use restarts::Restarts;

pub(super) const FALLBACK_GRID: (usize, usize) = (24, 80);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Placement {
    Tab,
    Split(Edge),
    Instance,
}

impl Running {
    pub(super) fn open(&mut self, placement: Placement) -> io::Result<()> {
        let cwd = self.start_folder(placement);
        let first = self.open_in(placement, cwd.as_deref(), None)?;
        if placement == Placement::Instance {
            for _ in 1..self.preferences.instance_tabs {
                self.open_in(Placement::Tab, cwd.as_deref(), None)?;
            }
            self.workspace.activate_tab(first);
            self.changed();
        }
        Ok(())
    }

    fn open_in(
        &mut self,
        placement: Placement,
        cwd: Option<&Path>,
        host: Option<SshHost>,
    ) -> io::Result<SessionId> {
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
        self.relayout();
        let spawned = self.start_shell(id, cwd, None);
        self.changed();
        spawned.map(|()| id)
    }

    pub(super) fn start_shell(
        &mut self,
        id: SessionId,
        cwd: Option<&Path>,
        revived: Option<&[u8]>,
    ) -> io::Result<()> {
        let start = ShellStart {
            cwd,
            size: self.grid_size_of(id).unwrap_or(FALLBACK_GRID),
            colors: self.theme.reported_colors(),
            transcript: revived.unwrap_or_default(),
        };
        let spawned = self.sessions.spawn(id, &start);
        match spawned {
            Ok(()) => self.greet(id, revived.is_none()),
            Err(_) => {
                self.workspace.close_tab(id);
            }
        }
        spawned
    }

    pub(super) fn open_or_report(&mut self, event_loop: &ActiveEventLoop, placement: Placement) {
        if let Err(error) = self.open(placement) {
            log_error!("the shell could not start: {error}");
            if self.workspace.is_empty() {
                event_loop.exit();
            }
        }
    }

    pub(super) fn close_tab(&mut self, event_loop: &ActiveEventLoop, id: SessionId) {
        if self.workspace.is_only_tab(id) && !event_loop.exiting() {
            self.replace_last_tab();
        }
        if self.workspace.close_tab(id) {
            self.sessions.remove(id);
            self.after_close(event_loop);
        }
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
