use crate::term::cell::{Cell, Flags};
use crate::term::grid::{Grid, Row};
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
            2 => {
                self.clear_rows(0, rows);
                self.clear_screen_images();
            }
            3 if !self.alt_active() => self.clear_history_images(),
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
        clear_cut_span(line, start, end, cols, &template);
        if end == cols {
            line.wrapped = false;
        }
    }

    pub(crate) fn erase_chars(&mut self, count: usize) {
        let (row, col, cols, template) = self.cursor_span();
        let end = col.saturating_add(count).min(cols);
        clear_cut_span(self.grid_mut().row_mut(row), col, end, cols, &template);
    }

    pub(crate) fn insert_chars(&mut self, count: usize) {
        let (row, col, cols, template) = self.cursor_span();
        let line = self.grid_mut().row_mut(row);
        let dropped = cols.saturating_sub(count).max(col);
        clear_cut_wide_char(line, col, cols, &template);
        clear_cut_wide_char(line, dropped, cols, &template);
        line.insert_blanks(col, count, cols, &template);
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn delete_chars(&mut self, count: usize) {
        let (row, col, cols, template) = self.cursor_span();
        let line = self.grid_mut().row_mut(row);
        clear_cut_wide_char(line, col, cols, &template);
        clear_cut_wide_char(line, col.saturating_add(count), cols, &template);
        line.delete_cells(col, count, cols, &template);
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

fn clear_cut_span(line: &mut Row, start: usize, end: usize, cols: usize, template: &Cell) {
    clear_cut_wide_char(line, start, cols, template);
    clear_cut_wide_char(line, end, cols, template);
    line.clear_range(start, end, template);
}

fn clear_cut_wide_char(line: &mut Row, edge: usize, cols: usize, template: &Cell) {
    let pair_cut = edge > 0
        && edge < cols
        && line.cell(edge - 1).flags.contains(Flags::WIDE)
        && line.cell(edge).flags.contains(Flags::WIDE_SPACER);
    if pair_cut {
        line.clear_range(edge - 1, edge + 1, template);
    }
}
