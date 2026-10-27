use std::io;

use winit::event_loop::ActiveEventLoop;

use crate::app::running::Running;
use crate::app::update;

impl Running {
    pub fn offer_update(&mut self, version: String) {
        self.update_offer = Some(version);
        self.changed();
    }

    pub(in crate::app::running) fn restart_for_update(&mut self, event_loop: &ActiveEventLoop) {
        let Some(offer) = self.update_offer.clone() else {
            return;
        };
        self.restart_with(
            event_loop,
            || update::restart(&offer),
            "the update restart did not start",
        );
    }

    pub(in crate::app::running) fn restart_if_card_lost(&mut self, event_loop: &ActiveEventLoop) {
        if self.restarting || !self.graphics.is_lost() {
            return;
        }
        self.restart_with(
            event_loop,
            update::start_again,
            "trame could not start again",
        );
    }

    fn restart_with(
        &mut self,
        event_loop: &ActiveEventLoop,
        start: impl FnOnce() -> io::Result<()>,
        failure: &str,
    ) {
        self.save_for_restart();
        match start() {
            Ok(()) => {
                self.restarting = true;
                event_loop.exit();
            }
            Err(error) => eprintln!("trame: {failure}: {error}"),
        }
    }
}
