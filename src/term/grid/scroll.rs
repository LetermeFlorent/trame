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

    pub fn clear_history(&mut self) {
        self.selection = None;
        let history = self.history_len();
        self.lines.drain(..history);
        self.lines.shrink_to_fit();
        self.dropped += history as u64;
        self.display_offset = 0;
    }

    pub fn reset(&mut self) {
        self.clear_history();
        for row in self.lines.iter_mut() {
            *row = Row::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Grid;
    use crate::term::Cell;

    #[test]
    fn clearing_the_history_gives_its_room_back() {
        let mut grid = Grid::new(4, 10, 1000);
        for _ in 0..500 {
            grid.scroll_up(0, 3, 1, &Cell::default());
        }
        assert_eq!(grid.history_len(), 500);
        grid.clear_history();
        assert!(grid.lines.capacity() < 64);
    }
}
