use crate::term::screen::Screen;

impl Screen {
    pub(crate) fn erase_display(&mut self, mode: u16) {
        let (row, rows, cols) = (self.cursor.row, self.rows(), self.cols());
        match mode {
            0 => {
                self.erase_line(0);
                self.clear_rows(row + 1, rows, cols);
            }
            1 => {
                self.erase_line(1);
                self.clear_rows(0, row, cols);
            }
            2 => self.clear_rows(0, rows, cols),
            3 => {
                if !self.alt_active() {
                    self.primary.clear_history();
                }
            }
            _ => {}
        }
    }

    pub(crate) fn erase_line(&mut self, mode: u16) {
        let (row, col, cols, template) = (
            self.cursor.row,
            self.cursor.col,
            self.cols(),
            self.cursor.template,
        );
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
        let (row, col, cols, template) = (
            self.cursor.row,
            self.cursor.col,
            self.cols(),
            self.cursor.template,
        );
        let end = col.saturating_add(count).min(cols);
        self.grid_mut()
            .row_mut(row)
            .clear_range(col, end, &template);
    }

    pub(crate) fn insert_chars(&mut self, count: usize) {
        let (row, col, cols, template) = (
            self.cursor.row,
            self.cursor.col,
            self.cols(),
            self.cursor.template,
        );
        self.grid_mut()
            .row_mut(row)
            .insert_blanks(col, count, cols, &template);
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn delete_chars(&mut self, count: usize) {
        let (row, col, cols, template) = (
            self.cursor.row,
            self.cursor.col,
            self.cols(),
            self.cursor.template,
        );
        self.grid_mut()
            .row_mut(row)
            .delete_cells(col, count, cols, &template);
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn insert_lines(&mut self, count: usize) {
        let (row, bottom, template) = (self.cursor.row, self.scroll_bottom, self.cursor.template);
        if row < self.scroll_top || row > bottom {
            return;
        }
        self.grid_mut().scroll_down(row, bottom, count, &template);
        self.carriage_return();
    }

    pub(crate) fn delete_lines(&mut self, count: usize) {
        let (row, bottom, template) = (self.cursor.row, self.scroll_bottom, self.cursor.template);
        if row < self.scroll_top || row > bottom {
            return;
        }
        self.grid_mut().rotate_up(row, bottom, count, &template);
        self.carriage_return();
    }

    fn clear_rows(&mut self, start: usize, end: usize, cols: usize) {
        let template = self.cursor.template;
        for row in start..end {
            let line = self.grid_mut().row_mut(row);
            line.clear_range(0, cols, &template);
            line.wrapped = false;
        }
    }
}
