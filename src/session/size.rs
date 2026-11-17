use super::Session;
use super::pty::PtySize;
use crate::log_error;

impl Session {
    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        let size = (rows.max(1), cols.max(1));
        if size == self.size {
            return;
        }
        self.size = size;
        self.terminal().screen_mut().resize(size.0, size.1);
        self.resize_pty();
    }

    pub fn set_cell_pixels(&mut self, width: u16, height: u16) {
        if (width, height) == self.cell_pixels {
            return;
        }
        self.cell_pixels = (width, height);
        self.terminal().screen_mut().set_cell_pixels(width, height);
        self.resize_pty();
    }

    fn resize_pty(&self) {
        let (rows, cols) = self.size;
        let (width, height) = self.cell_pixels;
        let size = PtySize::new(rows, cols)
            .with_pixels(cols * usize::from(width), rows * usize::from(height));
        if let Some(pty) = &self.pty
            && let Err(error) = pty.resize(size)
        {
            log_error!("resizing the terminal failed: {error}");
        }
    }
}
