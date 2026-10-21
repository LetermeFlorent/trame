use crate::term::cell::{Cell, Flags};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Row {
    cells: Vec<Cell>,
    pub wrapped: bool,
}

impl Row {
    pub fn blank(cols: usize, template: &Cell) -> Self {
        let blank = Cell::blank_with(template);
        if blank == Cell::default() {
            return Self::default();
        }
        Self {
            cells: vec![blank; cols],
            wrapped: false,
        }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn cell(&self, col: usize) -> Cell {
        self.cells.get(col).copied().unwrap_or_default()
    }

    pub fn cell_mut(&mut self, col: usize) -> &mut Cell {
        if col >= self.cells.len() {
            self.cells.resize(col + 1, Cell::default());
        }
        &mut self.cells[col]
    }

    pub fn is_blank(&self) -> bool {
        self.cells.iter().all(Cell::is_blank)
    }

    pub fn clear_range(&mut self, start: usize, end: usize, template: &Cell) {
        let blank = Cell::blank_with(template);
        if blank == Cell::default() && end >= self.cells.len() {
            self.cells.truncate(start.min(self.cells.len()));
            self.trim();
            return;
        }
        if end > self.cells.len() {
            self.cells.resize(end, Cell::default());
        }
        for cell in &mut self.cells[start.min(end)..end] {
            *cell = blank;
        }
    }

    pub fn insert_blanks(&mut self, col: usize, count: usize, cols: usize, template: &Cell) {
        if col >= cols {
            return;
        }
        if self.cells.len() < cols {
            self.cells.resize(cols, Cell::default());
        }
        let count = count.min(cols - col);
        self.cells[col..cols].rotate_right(count);
        self.clear_range(col, col + count, template);
    }

    pub fn delete_cells(&mut self, col: usize, count: usize, cols: usize, template: &Cell) {
        if col >= cols {
            return;
        }
        if self.cells.len() < cols {
            self.cells.resize(cols, Cell::default());
        }
        let count = count.min(cols - col);
        self.cells[col..cols].rotate_left(count);
        self.clear_range(cols - count, cols, template);
    }

    pub fn truncate(&mut self, cols: usize) {
        self.cells.truncate(cols);
    }

    pub fn trim(&mut self) {
        while self
            .cells
            .last()
            .is_some_and(|cell| *cell == Cell::default())
        {
            self.cells.pop();
        }
    }

    pub fn compact(&mut self) {
        self.trim();
        self.cells.shrink_to_fit();
    }

    pub fn text(&self) -> String {
        let mut text: String = self
            .cells
            .iter()
            .filter(|cell| !cell.flags.contains(Flags::WIDE_SPACER))
            .map(|cell| if cell.ch == '\0' { ' ' } else { cell.ch })
            .collect();
        let trimmed = text.trim_end().len();
        text.truncate(trimmed);
        text
    }
}
