use std::time::Instant;

use winit::event_loop::ActiveEventLoop;

use trame::preferences::Preferences;
use trame::theme::Rgba;

use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn sync_graphics(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.take_fresh() {
            self.changed();
        }
        self.idle.tick(Instant::now());
        self.restart_if_card_lost(event_loop);
    }

    pub(in crate::app::running) fn clear_color(&self) -> Rgba {
        self.theme.chrome
    }

    pub(in crate::app::running) fn apply_system(&mut self, previous: &Preferences) {
        let current = &self.preferences;
        self.workspace.set_numbering(current.numbering);
        if previous.efficiency != current.efficiency {
            self.idle.set_enabled(current.efficiency);
        }
    }
}
