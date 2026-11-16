use trame::host::quit::Quit;

use crate::app::running::Running;

impl Running {
    pub fn take_quit(&mut self) -> Option<Quit> {
        self.quit.take()
    }

    pub(in crate::app::running) fn quit_with(&mut self, quit: Quit) {
        if !matches!(self.quit, Some(Quit::Restart(_))) {
            self.quit = Some(quit);
        }
    }

    pub(in crate::app::running) fn quitting(&self) -> bool {
        self.quit.is_some()
    }
}
