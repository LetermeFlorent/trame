use std::time::Instant;

use trame::workspace::Snapshot;

use crate::app::running::Running;
use crate::app::window;

impl Running {
    pub(super) fn save_deadline(&mut self) -> Option<Instant> {
        let save = self
            .desk
            .save_deadline(Instant::now(), || window::snapshot(&self.window));
        self.apply(save.effects);
        save.value
    }

    pub fn save_on_exit(&mut self) {
        let effects = self.desk.save_on_exit(window::snapshot(&self.window));
        self.apply(effects);
    }

    pub fn save_for_restart(&mut self) {
        let effects = self.desk.save_for_restart(window::snapshot(&self.window));
        self.apply(effects);
    }

    pub(super) fn restore_session(&mut self, saved: Option<&Snapshot>) -> bool {
        if !self.desk.restore(saved) {
            return false;
        }
        self.relayout();
        self.start_shown();
        if !self.desk.settle_restore() {
            return false;
        }
        self.changed();
        true
    }
}
