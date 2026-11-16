use trame::host::quit::Quit;

use crate::app::running::Running;

impl Running {
    pub fn take_quit(&mut self) -> Option<Quit> {
        self.desk.take_quit()
    }
}
