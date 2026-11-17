use crate::host::quit::Quit;

use super::Desk;

impl Desk {
    pub fn take_quit(&mut self) -> Option<Quit> {
        self.quit.take()
    }

    pub fn quit_with(&mut self, quit: Quit) {
        if !matches!(self.quit, Some(Quit::Restart(_))) {
            self.quit = Some(quit);
        }
    }

    pub fn quitting(&self) -> bool {
        self.quit.is_some()
    }
}
