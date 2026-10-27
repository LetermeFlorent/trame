use super::{Grid, Row};

impl Grid {
    pub fn resize(&mut self, rows: usize, cols: usize, cursor_row: &mut usize) {
        self.selection = None;
        let rows = rows.max(1);
        let cols = cols.max(1);
        if cols < self.cols {
            let start = self.screen_start();
            for row in self.lines.iter_mut().skip(start) {
                row.truncate(cols);
            }
        }
        self.cols = cols;
        if rows < self.rows {
            self.shrink_rows(self.rows - rows, cursor_row);
        } else if rows > self.rows {
            self.lines
                .resize_with(self.lines.len() + rows - self.rows, Row::default);
            self.rows = rows;
        }
        *cursor_row = (*cursor_row).min(self.rows - 1);
        self.enforce_history_limit();
    }

    pub fn compact(&mut self) {
        for row in self.lines.iter_mut() {
            row.compact();
        }
        self.lines.shrink_to_fit();
    }

    fn shrink_rows(&mut self, mut excess: usize, cursor_row: &mut usize) {
        while excess > 0 && self.rows - 1 > *cursor_row && self.row(self.rows - 1).is_blank() {
            self.lines.pop_back();
            self.rows -= 1;
            excess -= 1;
        }
        if excess == 0 {
            return;
        }
        self.rows -= excess;
        *cursor_row = cursor_row.saturating_sub(excess);
        if self.max_history == 0 {
            self.lines.drain(..excess);
            self.dropped += excess as u64;
            return;
        }
        let start = self.screen_start();
        for row in self.lines.range_mut(start - excess..start) {
            row.compact();
        }
    }
}
