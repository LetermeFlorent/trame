use std::io;

use winit::event_loop::ActiveEventLoop;

use trame::log_error;

use super::running::{Restart, Running};
use super::{App, update};

impl App {
    pub(super) fn restart(
        &mut self,
        event_loop: &ActiveEventLoop,
        start: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<()> {
        self.windows.iter_mut().for_each(Running::save_for_restart);
        self.role.hand_over(start)?;
        self.windows.iter_mut().for_each(Running::mark_restarting);
        event_loop.exit();
        Ok(())
    }

    pub(super) fn restart_for(&mut self, event_loop: &ActiveEventLoop, restart: Restart) {
        if event_loop.exiting() {
            return;
        }
        let start = match restart {
            Restart::Update => update::restart,
            Restart::Again => update::start_again,
        };
        let Err(error) = self.restart(event_loop, start) else {
            return;
        };
        log_error!("trame could not start again: {error}");
        if restart == Restart::Update {
            self.windows.iter_mut().for_each(Running::update_failed);
        }
    }
}
