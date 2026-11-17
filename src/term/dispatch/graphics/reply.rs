use crate::term::graphics::{Command, Error, reply, wanted};
use crate::term::screen::Screen;

impl Screen {
    pub(in crate::term::dispatch) fn graphics_reply(
        &mut self,
        command: &Command,
        id: u32,
        result: Result<(), Error>,
    ) {
        let addressed = command.id != 0 || command.number != 0;
        if !addressed || !wanted(command.quiet, result.is_ok()) {
            return;
        }
        let bytes = reply(id, command.number, command.placement, result);
        self.respond_string(&String::from_utf8_lossy(&bytes));
    }

    pub(in crate::term::dispatch) fn graphics_id(&self, command: &Command) -> u32 {
        match (command.id, command.number) {
            (0, 0) => 0,
            (0, number) => self.graphics.by_number(number).unwrap_or(0),
            (id, _) => id,
        }
    }
}
