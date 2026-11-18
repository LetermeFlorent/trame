use super::args::parse;
use crate::term::graphics::MAX_ENCODED;
use crate::term::screen::Screen;

impl Screen {
    pub(super) fn start_multipart(&mut self, args: &[u8]) {
        let args = parse(args);
        self.multipart = args.inline.then(|| (args, Vec::new()));
    }

    pub(super) fn add_file_part(&mut self, part: &[u8]) {
        let Some((_, data)) = self.multipart.as_mut() else {
            return;
        };
        if data.len() + part.len() > MAX_ENCODED {
            self.multipart = None;
            return;
        }
        data.extend_from_slice(part);
    }

    pub(super) fn end_multipart(&mut self) {
        if let Some((args, data)) = self.multipart.take() {
            self.show_file(&args, &data);
        }
    }
}
