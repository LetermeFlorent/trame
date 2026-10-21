use super::Screen;
use crate::term::grid::{GridPoint, Selection, SelectionKind};

impl Screen {
    pub fn selection(&self) -> Option<&Selection> {
        self.grid().selection()
    }

    pub fn selected_text(&self) -> Option<String> {
        self.grid().selected_text()
    }

    pub fn clear_selection(&mut self) {
        self.grid_mut().clear_selection();
    }

    pub fn start_selection(&mut self, kind: SelectionKind, row: usize, col: usize) {
        let at = self.visible_point(row, col);
        self.grid_mut().start_selection(kind, at);
    }

    pub fn extend_selection(&mut self, row: usize, col: usize) {
        let to = self.visible_point(row, col);
        self.grid_mut().extend_selection(to);
    }

    fn visible_point(&self, row: usize, col: usize) -> GridPoint {
        let row = row.min(self.rows() - 1);
        GridPoint::new(self.grid().first_visible_line() + row as u64, col)
    }
}
