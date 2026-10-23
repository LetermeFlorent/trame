mod resize;
mod row;
mod scroll;
mod select;
mod selection;
mod text;
mod words;

use std::collections::VecDeque;

pub use row::Row;
pub use selection::{GridPoint, Selection, SelectionKind, Span};

#[derive(Debug, Clone)]
pub struct Grid {
    lines: VecDeque<Row>,
    rows: usize,
    cols: usize,
    max_history: usize,
    dropped: u64,
    display_offset: usize,
    selection: Option<Selection>,
}

impl Grid {
    pub fn new(rows: usize, cols: usize, max_history: usize) -> Self {
        let rows = rows.max(1);
        let mut lines = VecDeque::with_capacity(rows);
        lines.resize_with(rows, Row::default);
        Self {
            lines,
            rows,
            cols: cols.max(1),
            max_history,
            dropped: 0,
            display_offset: 0,
            selection: None,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn history_len(&self) -> usize {
        self.lines.len() - self.rows
    }

    pub fn display_offset(&self) -> usize {
        self.display_offset
    }

    pub fn scroll_display(&mut self, delta: isize) {
        let target = self.display_offset as isize + delta;
        self.display_offset = target.clamp(0, self.history_len() as isize) as usize;
    }

    pub fn reset_display(&mut self) {
        self.display_offset = 0;
    }

    pub fn row(&self, row: usize) -> &Row {
        &self.lines[self.screen_start() + row]
    }

    pub fn row_mut(&mut self, row: usize) -> &mut Row {
        let line = self.screen_line(row);
        self.drop_selection_touching(line, line);
        let start = self.screen_start();
        &mut self.lines[start + row]
    }

    pub fn visible_row(&self, row: usize) -> &Row {
        &self.lines[self.screen_start() - self.display_offset + row]
    }

    pub fn view_row(&self, row: isize) -> Option<&Row> {
        let start = (self.screen_start() - self.display_offset) as isize;
        let index = usize::try_from(start + row).ok()?;
        self.lines.get(index)
    }

    pub fn first_visible_line(&self) -> u64 {
        self.dropped + (self.screen_start() - self.display_offset) as u64
    }

    pub fn first_line(&self) -> u64 {
        self.dropped
    }

    pub fn end_line(&self) -> u64 {
        self.dropped + self.lines.len() as u64
    }

    pub fn line(&self, absolute: u64) -> Option<&Row> {
        let index = absolute.checked_sub(self.dropped)?;
        self.lines.get(usize::try_from(index).ok()?)
    }

    pub fn screen_line(&self, row: usize) -> u64 {
        self.dropped + (self.screen_start() + row) as u64
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

    fn screen_start(&self) -> usize {
        self.lines.len() - self.rows
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
