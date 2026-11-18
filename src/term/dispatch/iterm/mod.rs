mod args;
mod cell_size;
mod cells;
mod file;
mod multipart;

pub(crate) use args::FileArgs;

use crate::term::screen::Screen;

const FILE: &[u8] = b"File=";
const MULTIPART: &[u8] = b"MultipartFile=";
const PART: &[u8] = b"FilePart=";
const END: &[u8] = b"FileEnd";
const CELL_SIZE: &[u8] = b"ReportCellSize";

impl Screen {
    pub(super) fn iterm(&mut self, params: &[&[u8]], bell_terminated: bool) {
        let Some(&first) = params.first() else {
            return;
        };
        let body = |prefix: &[u8]| params.join(&b';').split_off(prefix.len());
        if first.starts_with(FILE) {
            self.iterm_file(&body(FILE));
        } else if first.starts_with(MULTIPART) {
            self.start_multipart(&body(MULTIPART));
        } else if let Some(part) = first.strip_prefix(PART) {
            self.add_file_part(part);
        } else if first == END {
            self.end_multipart();
        } else if first == CELL_SIZE {
            self.report_cell_size(bell_terminated);
        }
    }
}
