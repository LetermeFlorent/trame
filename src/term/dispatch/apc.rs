use crate::term::graphics::{Action, Error, parse, split};
use crate::term::screen::Screen;

const ACTION_KEY: &[u8] = b"a=";

impl Screen {
    pub(crate) fn apc(&mut self, data: &[u8]) {
        let Some((control, payload)) = split(data) else {
            return;
        };
        let command = parse(control);
        if let Some(pending) = self.graphics.pending.take()
            && !names_action(control)
        {
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

fn names_action(control: &[u8]) -> bool {
    control
        .split(|&byte| byte == b',')
        .any(|pair| pair.starts_with(ACTION_KEY))
}
