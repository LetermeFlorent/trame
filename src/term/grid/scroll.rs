use super::{Grid, Row};
use crate::term::cell::Cell;

impl Grid {
    pub fn scroll_up(&mut self, top: usize, bottom: usize, count: usize, template: &Cell) {
        if top > bottom || bottom >= self.rows {
            return;
        }
        let count = count.min(bottom - top + 1);
        if top == 0 && bottom == self.rows - 1 && self.max_history > 0 {
            self.push_into_history(count, template);
            return;
        }
        self.rotate_up(top, bottom, count, template);
    }

    pub fn rotate_up(&mut self, top: usize, bottom: usize, count: usize, template: &Cell) {
        if let Some(start) = self.open_region(top, bottom) {
            let count = count.min(bottom - top + 1);
            self.cycle_rows(start + top, start + bottom, count, template);
        }
    }

    pub fn scroll_down(&mut self, top: usize, bottom: usize, count: usize, template: &Cell) {
        if let Some(start) = self.open_region(top, bottom) {
            let count = count.min(bottom - top + 1);
            self.cycle_rows(start + bottom, start + top, count, template);
        }
    }

    fn open_region(&mut self, top: usize, bottom: usize) -> Option<usize> {
        if top > bottom || bottom >= self.rows {
            return None;
        }
        self.drop_selection_touching(self.screen_line(top), self.screen_line(bottom));
        Some(self.screen_start())
    }

    fn cycle_rows(&mut self, removed: usize, inserted: usize, count: usize, template: &Cell) {
        for _ in 0..count {
            self.lines.remove(removed);
            self.lines.insert(inserted, Row::blank(self.cols, template));
        }
    }

    fn push_into_history(&mut self, count: usize, template: &Cell) {
        for _ in 0..count {
            let leaving = self.screen_start();
            self.lines[leaving].compact();
            self.lines.push_back(Row::blank(self.cols, template));
            if self.display_offset > 0 {
                self.display_offset += 1;
            }
        }
        self.enforce_history_limit();
    }

    pub(super) fn enforce_history_limit(&mut self) {
        while self.history_len() > self.max_history {
            self.lines.pop_front();
            self.dropped += 1;
        }
        self.display_offset = self.display_offset.min(self.history_len());
    }
}
