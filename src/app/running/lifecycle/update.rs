use winit::event_loop::ActiveEventLoop;

use crate::app::running::Running;
use crate::app::update;

impl Running {
    pub fn offer_update(&mut self, version: String) {
        self.update_offer = Some(version);
        self.changed();
    }

    pub(in crate::app::running) fn restart_for_update(&mut self, event_loop: &ActiveEventLoop) {
        self.save_for_restart();
        match update::restart() {
            Ok(()) => {
                self.restarting = true;
                event_loop.exit();
            }
            Err(error) => eprintln!("trame: the update restart did not start: {error}"),
        }
    }
}
