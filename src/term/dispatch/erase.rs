use crate::term::cell::Cell;
use crate::term::grid::Grid;
use crate::term::screen::Screen;

type LineShift = fn(&mut Grid, usize, usize, usize, &Cell);

impl Screen {
    pub(crate) fn erase_display(&mut self, mode: u16) {
        let (row, rows) = (self.cursor.row, self.rows());
        match mode {
            0 => {
                self.erase_line(0);
                self.clear_rows(row + 1, rows);
            }
            1 => {
                self.erase_line(1);
                self.clear_rows(0, row);
            }
            2 => self.clear_rows(0, rows),
            3 if !self.alt_active() => self.primary.clear_history(),
            _ => {}
        }
    }

    pub(crate) fn erase_line(&mut self, mode: u16) {
        let (row, col, cols, template) = self.cursor_span();
        let (start, end) = match mode {
            0 => (col, cols),
            1 => (0, col + 1),
            2 => (0, cols),
            _ => return,
        };
        let line = self.grid_mut().row_mut(row);
        line.clear_range(start, end, &template);
        if end == cols {
            line.wrapped = false;
        }
    }

    pub(crate) fn erase_chars(&mut self, count: usize) {
        let (row, col, cols, template) = self.cursor_span();
        let end = col.saturating_add(count).min(cols);
        self.grid_mut()
            .row_mut(row)
            .clear_range(col, end, &template);
    }

    pub(crate) fn insert_chars(&mut self, count: usize) {
        let (row, col, cols, template) = self.cursor_span();
        self.grid_mut()
            .row_mut(row)
            .insert_blanks(col, count, cols, &template);
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn delete_chars(&mut self, count: usize) {
        let (row, col, cols, template) = self.cursor_span();
        self.grid_mut()
            .row_mut(row)
            .delete_cells(col, count, cols, &template);
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn insert_lines(&mut self, count: usize) {
        self.shift_lines(count, Grid::scroll_down);
    }

    pub(crate) fn delete_lines(&mut self, count: usize) {
        self.shift_lines(count, Grid::rotate_up);
    }

    fn shift_lines(&mut self, count: usize, shift: LineShift) {
        let (row, bottom, template) = (self.cursor.row, self.scroll_bottom, self.cursor.template);
        if row < self.scroll_top || row > bottom {
            return;
        }
        shift(self.grid_mut(), row, bottom, count, &template);
        self.carriage_return();
    }

    fn cursor_span(&self) -> (usize, usize, usize, Cell) {
        (
            self.cursor.row,
            self.cursor.col,
            self.cols(),
            self.cursor.template,
        )
    }

    fn clear_rows(&mut self, start: usize, end: usize) {
        let (template, cols) = (self.cursor.template, self.cols());
        for row in start..end {
            let line = self.grid_mut().row_mut(row);
            line.clear_range(0, cols, &template);
            line.wrapped = false;
        }
    }
}
