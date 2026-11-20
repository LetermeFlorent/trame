use adw::prelude::*;
use trame::host::pending::Offer;
use trame::host::quit::Restart;
use trame::host::update::{self, UpdateStatus};
use trame::log_error;

use super::Fleet;

impl Fleet {
    pub fn updated(&mut self, app: &adw::Application, status: UpdateStatus) {
        match status {
            UpdateStatus::Installed(handoff) => {
                if let Err(err) = self.do_restart(app, || handoff.start()) {
                    log_error!("the new version did not start: {err}");
                    self.broadcast_failed();
                }
            }
            UpdateStatus::AvailableAtStart(v) if self.auto_update_enabled() => {
                self.offer.send(Offer { version: v }, &mut self.windows);
                self.restart_for(app, Restart::Update);
            }
            UpdateStatus::Available(v) | UpdateStatus::AvailableAtStart(v) => {
                self.offer.send(Offer { version: v }, &mut self.windows);
            }
            UpdateStatus::Failed => self.broadcast_failed(),
            other => self.update = other,
        }
    }

    fn auto_update_enabled(&self) -> bool {
        self.windows
            .first()
            .is_some_and(|w| w.state.borrow().desk.preferences.update_on_start)
    }

    fn broadcast_failed(&mut self) {
        self.update = UpdateStatus::Failed;
        for w in &self.windows {
            let mut s = w.state.borrow_mut();
            let effects = s.desk.update_failed();
            s.apply(effects);
            drop(s);
            (w.refresh)();
        }
    }

    pub(super) fn restart_for(&mut self, app: &adw::Application, restart: Restart) {
        if self.exiting {
            return;
        }
        let start: fn() -> std::io::Result<()> = match restart {
            Restart::Update => update::restart,
            Restart::Again => update::start_again,
        };
        if let Err(err) = self.do_restart(app, start) {
            log_error!("trame could not start again: {err}");
            if restart == Restart::Update {
                self.broadcast_failed();
            }
        }
    }

    fn do_restart(
        &mut self,
        app: &adw::Application,
        start: impl FnOnce() -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        for w in &self.windows {
            let mut s = w.state.borrow_mut();
            let snap = s.snapshot();
            let effects = s.desk.save_for_restart(snap);
            s.apply(effects);
        }
        self.role.hand_over(start)?;
        for w in &self.windows {
            w.state.borrow_mut().desk.mark_restarting();
        }
        self.exiting = true;
        app.quit();
        Ok(())
    }
}
