use crate::term::screen::Screen;

const WINDOW_PIXELS_REQUEST: u16 = 14;
const CELL_PIXELS_REQUEST: u16 = 16;
const WINDOW_SIZE_REQUEST: u16 = 18;

impl Screen {
    pub(super) fn report_window(&mut self, kind: u16) {
        let (width, height) = self.cell_pixels;
        let known = width > 0 && height > 0;
        let (rows, cols) = (self.rows(), self.cols());
        let report = match kind {
            WINDOW_PIXELS_REQUEST if known => {
                let total_height = rows * usize::from(height);
                let total_width = cols * usize::from(width);
                format!("\x1b[4;{total_height};{total_width}t")
            }
            CELL_PIXELS_REQUEST if known => format!("\x1b[6;{height};{width}t"),
            WINDOW_SIZE_REQUEST => format!("\x1b[8;{rows};{cols}t"),
            _ => return,
        };
        self.respond(report.as_bytes());
    }
}
