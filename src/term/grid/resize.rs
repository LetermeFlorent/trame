use super::{Grid, Row};

impl Grid {
    pub fn resize(&mut self, rows: usize, cols: usize, cursor_row: &mut usize) {
        self.selection = None;
        let rows = rows.max(1);
        self.cols = cols.max(1);
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
        let pushed = self.screen_start() - excess;
        let doomed = self.history_len().saturating_sub(self.max_history);
        let recyclable = doomed.min(pushed);
        for (oldest, index) in (pushed.max(doomed)..pushed + excess).enumerate() {
            if oldest < recyclable {
                let mut kept = std::mem::take(&mut self.lines[oldest]);
                kept.refill_from(&mut self.lines[index]);
                self.lines[index] = kept;
            } else {
                self.lines[index].compact();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::term::{Cell, Terminal};

    fn buffer(terminal: &Terminal, line: u64) -> Option<*const Cell> {
        terminal
            .screen()
            .grid()
            .line(line)
            .map(|row| row.cells().as_ptr())
    }

    fn every_line(terminal: &Terminal) -> Vec<(String, bool)> {
        let grid = terminal.screen().grid();
        (grid.first_line()..grid.end_line())
            .filter_map(|line| grid.line(line))
            .map(|row| (row.text(), row.wrapped))
            .collect()
    }

    fn unwrapped(texts: &[&str]) -> Vec<(String, bool)> {
        texts.iter().map(|text| (text.to_string(), false)).collect()
    }

    #[test]
    fn a_shrink_into_a_full_history_keeps_the_rows_it_pushes_and_drops_the_oldest() {
        let mut terminal = Terminal::new(4, 5, 3);
        terminal.feed("h1\r\nh2\r\nh3\r\ne\r\nwrappe\u{301}d\r\nc\r\nd".as_bytes());
        let dropped_buffer = buffer(&terminal, 2);
        terminal.screen_mut().resize(2, 5);
        let mut expected = unwrapped(&["e", "wrapp", "e\u{301}d", "c", "d"]);
        expected[1].1 = true;
        assert_eq!(every_line(&terminal), expected);
        assert_eq!(terminal.screen().grid().first_line(), 3);
        assert_eq!(buffer(&terminal, 5), dropped_buffer);
    }

    #[test]
    fn a_shrink_deeper_than_the_history_keeps_only_the_newest_rows() {
        let mut terminal = Terminal::new(4, 5, 1);
        terminal.feed("a\r\nb\r\nc\r\nd\u{301}\r\ne".as_bytes());
        let dropped_buffer = buffer(&terminal, 0);
        terminal.screen_mut().resize(1, 5);
        assert_eq!(every_line(&terminal), unwrapped(&["d\u{301}", "e"]));
        assert_eq!(terminal.screen().grid().first_line(), 3);
        assert_eq!(buffer(&terminal, 3), dropped_buffer);
    }
}
