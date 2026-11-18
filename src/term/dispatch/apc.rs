use crate::term::graphics::{Action, Command, Error, parse, split};
use crate::term::screen::Screen;

const QUIET_KEY: &[u8] = b"q=";

impl Screen {
    pub(crate) fn apc(&mut self, data: &[u8]) {
        let Some((control, payload)) = split(data) else {
            return;
        };
        let command = parse(control);
        if let Some(mut pending) = self.graphics.pending.take()
            && continues(&command, pending.first())
        {
            if names_key(control, QUIET_KEY) {
                pending.set_quiet(command.quiet);
            }
            self.continue_stream(pending, command.more, payload);
            return;
        }
        match command.action {
            Action::Transmit | Action::TransmitPlace | Action::Query => {
                self.start_stream(command, payload);
            }
            Action::Place => {
                let id = self.graphics_id(&command);
                self.place(&command, id);
            }
            Action::Delete => self.delete_images(&command),
            Action::Unsupported => self.graphics_reply(&command, command.id, Err(Error::Inval)),
        }
    }
}

fn continues(command: &Command, first: &Command) -> bool {
    let same = |key: u32, pending: u32| key == 0 || key == pending;
    matches!(
        command.action,
        Action::Transmit | Action::TransmitPlace | Action::Query
    ) && same(command.id, first.id)
        && same(command.number, first.number)
}

fn names_key(control: &[u8], key: &[u8]) -> bool {
    control
        .split(|&byte| byte == b',')
        .any(|pair| pair.starts_with(key))
}
