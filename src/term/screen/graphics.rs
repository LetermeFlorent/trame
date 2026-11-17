use super::Screen;
use crate::term::graphics::ImageStore;

impl Screen {
    pub fn graphics(&self) -> &ImageStore {
        &self.graphics
    }

    pub fn cell_pixels(&self) -> (u16, u16) {
        self.cell_pixels
    }

    pub fn set_cell_pixels(&mut self, width: u16, height: u16) {
        self.cell_pixels = (width, height);
    }
}
