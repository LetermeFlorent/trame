use crate::term::graphics::{Action, Chunks, Command, Error, ImageStore, decode};
use crate::term::screen::Screen;

const DIRECT: u8 = b'd';

impl Screen {
    pub(in crate::term::dispatch) fn start_stream(&mut self, command: Command, payload: &[u8]) {
        if command.id != 0 && command.number != 0 {
            self.graphics_reply(&command, command.id, Err(Error::Inval));
            return;
        }
        if command.transport != DIRECT {
            self.graphics_reply(&command, command.id, Err(Error::BadF));
            return;
        }
        let more = command.more;
        self.continue_stream(Chunks::start(command), more, payload);
    }

    pub(in crate::term::dispatch) fn continue_stream(
        &mut self,
        mut stream: Chunks,
        more: bool,
        payload: &[u8],
    ) {
        match stream.push(more, payload) {
            Ok(None) => self.graphics.pending = Some(stream),
            Ok(Some((command, raw))) => self.finish_stream(&command, raw),
            Err(error) => {
                let first = stream.first().clone();
                self.graphics_reply(&first, first.id, Err(error));
            }
        }
    }

    fn finish_stream(&mut self, command: &Command, raw: Vec<u8>) {
        let (width, height, pixels) = match decode(command, raw) {
            Ok(decoded) => decoded,
            Err(error) => return self.graphics_reply(command, command.id, Err(error)),
        };
        if command.action == Action::Query {
            return self.graphics_reply(command, command.id, Ok(()));
        }
        let id = match command.id {
            0 => self.graphics.assign(command.number),
            id => id,
        };
        self.replace_image(id);
        self.graphics.insert(id, width, height, pixels);
        self.graphics
            .evict_unreferenced(ImageStore::QUOTA, [&self.primary, &self.alternate]);
        self.graphics_reply(command, id, Ok(()));
        if command.action == Action::TransmitPlace {
            self.place(command, id);
        }
    }
}
