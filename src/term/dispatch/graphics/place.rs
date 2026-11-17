use crate::term::graphics::Command;
use crate::term::screen::Screen;

impl Screen {
    pub(in crate::term::dispatch) fn place(&mut self, _command: &Command, _id: u32) {}
}
