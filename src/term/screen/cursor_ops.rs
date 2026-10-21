use super::Screen;
use crate::term::cursor::SavedCursor;
use crate::term::modes::Modes;

impl Screen {
    pub(crate) fn goto(&mut self, row: usize, col: usize) {
        let origin = self.modes.contains(Modes::ORIGIN);
        let (top, bottom) = if origin {
            (self.scroll_top, self.scroll_bottom)
        } else {
            (0, self.rows() - 1)
        };
        let row = if origin { row.saturating_add(top) } else { row };
        self.cursor.row = row.clamp(top, bottom);
        self.cursor.col = col.min(self.cols() - 1);
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn goto_row(&mut self, row: usize) {
        let col = self.cursor.col;
        self.goto(row, col);
    }

    pub(crate) fn goto_col(&mut self, col: usize) {
        self.cursor.col = col.min(self.cols() - 1);
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn move_up(&mut self, count: usize) {
        let limit = if self.cursor.row >= self.scroll_top {
            self.scroll_top
        } else {
            0
        };
        self.cursor.row = self.cursor.row.saturating_sub(count).max(limit);
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn move_down(&mut self, count: usize) {
        let limit = if self.cursor.row <= self.scroll_bottom {
            self.scroll_bottom
        } else {
            self.rows() - 1
        };
        self.cursor.row = self.cursor.row.saturating_add(count).min(limit);
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn move_forward(&mut self, count: usize) {
        self.goto_col(self.cursor.col.saturating_add(count));
    }

    pub(crate) fn move_backward(&mut self, count: usize) {
        self.goto_col(self.cursor.col.saturating_sub(count));
    }

    pub(crate) fn save_cursor(&mut self) {
        let slot = usize::from(self.alt_active());
        self.saved[slot] = SavedCursor {
            row: self.cursor.row,
            col: self.cursor.col,
            template: self.cursor.template,
            pending_wrap: self.cursor.pending_wrap,
            origin: self.modes.contains(Modes::ORIGIN),
            charsets: self.charsets,
        };
    }

    pub(crate) fn restore_cursor(&mut self) {
        let saved = self.saved[usize::from(self.alt_active())];
        self.cursor.row = saved.row.min(self.rows() - 1);
        self.cursor.col = saved.col.min(self.cols() - 1);
        self.cursor.template = saved.template;
        self.cursor.pending_wrap = saved.pending_wrap;
        self.modes.set(Modes::ORIGIN, saved.origin);
        self.charsets = saved.charsets;
    }

    pub(crate) fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        let last = self.rows() - 1;
        let bottom = if bottom == 0 {
            last
        } else {
            (bottom - 1).min(last)
        };
        let top = top.saturating_sub(1);
        if top >= bottom {
            return;
        }
        self.scroll_top = top;
        self.scroll_bottom = bottom;
        self.goto(0, 0);
    }
}
