use super::Screen;
use crate::term::cursor::CursorShape;
use crate::term::grid::Grid;
use crate::term::modes::Modes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportedColors {
    pub foreground: [u8; 3],
    pub background: [u8; 3],
    pub cursor: [u8; 3],
}

impl Default for ReportedColors {
    fn default() -> Self {
        Self {
            foreground: [0xDC, 0xE0, 0xE5],
            background: [0x28, 0x2C, 0x33],
            cursor: [0x74, 0xAD, 0xE8],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorView {
    pub row: usize,
    pub col: usize,
    pub visible: bool,
    pub shape: CursorShape,
}

impl Screen {
    pub fn rows(&self) -> usize {
        self.grid().rows()
    }

    pub fn cols(&self) -> usize {
        self.grid().cols()
    }

    pub fn grid(&self) -> &Grid {
        if self.alt_active() {
            &self.alternate
        } else {
            &self.primary
        }
    }

    pub fn modes(&self) -> Modes {
        self.modes
    }

    pub fn cursor(&self) -> CursorView {
        CursorView {
            row: self.cursor.row,
            col: self.cursor.col,
            visible: self.modes.contains(Modes::CURSOR_VISIBLE),
            shape: self.cursor_shape,
        }
    }

    pub fn cursor_line(&self) -> String {
        let grid = self.grid();
        let last = self.cursor.row.min(grid.rows().saturating_sub(1));
        let first = (0..last)
            .rev()
            .take_while(|&row| grid.row(row).wrapped)
            .last()
            .unwrap_or(last);
        (first..=last).map(|row| grid.row(row).text()).collect()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn cwd(&self) -> Option<&str> {
        self.cwd.as_deref()
    }

    pub fn take_responses(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.responses)
    }

    pub fn take_clipboard(&mut self) -> Option<String> {
        self.clipboard.take()
    }

    pub fn take_attention(&mut self) -> Option<String> {
        self.attention.take()
    }

    pub fn scroll_display(&mut self, delta: isize) {
        self.grid_mut().scroll_display(delta);
    }

    pub fn reset_display(&mut self) {
        self.grid_mut().reset_display();
    }

    pub fn set_reported_colors(&mut self, colors: ReportedColors) {
        self.colors = colors;
    }

    pub fn set_history(&mut self, lines: usize) {
        self.primary.set_max_history(lines);
    }

    pub fn compact(&mut self) {
        self.primary.compact();
        self.alternate.compact();
        self.responses.shrink_to_fit();
    }
}
