use super::{Screen, default_tab_stops};

impl Screen {
    pub fn resize(&mut self, rows: usize, cols: usize) {
        let rows = rows.max(1);
        let cols = cols.max(1);
        if rows == self.rows() && cols == self.cols() {
            return;
        }
        let alt = self.alt_active();
        let mut primary_row = if alt {
            self.saved[0].row
        } else {
            self.cursor.row
        };
        self.primary.resize(rows, cols, &mut primary_row);
        let mut alternate_row = if alt { self.cursor.row } else { 0 };
        self.alternate.resize(rows, cols, &mut alternate_row);
        if alt {
            self.cursor.row = alternate_row;
            self.saved[0].row = primary_row;
        } else {
            self.cursor.row = primary_row;
        }
        self.cursor.col = self.cursor.col.min(cols - 1);
        self.cursor.pending_wrap = false;
        for saved in &mut self.saved {
            saved.row = saved.row.min(rows - 1);
            saved.col = saved.col.min(cols - 1);
        }
        self.scroll_top = 0;
        self.scroll_bottom = rows - 1;
        self.tab_stops = default_tab_stops(cols);
    }
}
