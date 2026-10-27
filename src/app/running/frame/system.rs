use std::collections::HashSet;
use std::time::Instant;

use winit::event_loop::ActiveEventLoop;

use trame::preferences::Preferences;
use trame::session::SessionId;
use trame::theme::Rgba;

use crate::app::power::Shown;
use crate::app::running::Running;
use crate::app::update;

impl Running {
    pub(in crate::app::running) fn sync_graphics(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.take_fresh() {
            self.changed();
        }
        self.power.tick(Instant::now());
        self.compact_rested();
        self.restart_if_card_lost(event_loop);
    }

    pub(in crate::app::running) fn sync_rest(&mut self) {
        let shown: HashSet<SessionId> = self
            .frame
            .panes
            .iter()
            .filter_map(|pane| pane.active)
            .collect();
        let tabs = self.sessions.map().iter().filter_map(|(&id, session)| {
            let shell = session.shell_id()?;
            Some(Shown {
                id,
                shell,
                shown: shown.contains(&id),
            })
        });
        self.power.set_tabs(tabs);
    }

    fn compact_rested(&mut self) {
        for id in self.power.take_rested() {
            if let Some(session) = self.sessions.get(id) {
                session.terminal().compact();
            }
        }
    }

    pub(in crate::app::running) fn clear_color(&self) -> Rgba {
        self.theme.chrome
    }

    pub(in crate::app::running) fn apply_system(&mut self, previous: &Preferences) {
        let current = &self.preferences;
        self.workspace.set_numbering(current.numbering);
        let wanted = current.power_switches();
        if previous.power_switches() != wanted {
            self.power.set_enabled(wanted);
        }
        update::download_ahead(current.updates.downloads_ahead());
    }
}
