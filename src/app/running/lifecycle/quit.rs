use crate::app::running::Running;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Restart {
    Update,
    Again,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quit {
    Close,
    Restart(Restart),
}

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
