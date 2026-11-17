use super::{CursorPlace, drawn_rows};
use crate::term::Screen;
use crate::term::grid::{Row, Selection};

pub struct ScreenCopy {
    pub(super) rows: Vec<(isize, Row)>,
    pub(super) cols: usize,
    pub(super) cursor: Option<CursorPlace>,
    selection: Option<Selection>,
    first_line: u64,
}

impl ScreenCopy {
    pub fn of(screen: &Screen, scroll: f32) -> Self {
        let grid = screen.grid();
        Self {
            rows: drawn_rows(screen, scroll)
                .filter_map(|row| Some((row, grid.view_row(row)?.clone())))
                .collect(),
            cols: grid.cols(),
            cursor: CursorPlace::locate(screen),
            selection: grid.selection().cloned(),
            first_line: grid.first_visible_line(),
        }
    }

    pub fn is_selected(&self, row: isize, col: usize) -> bool {
        let line = self.first_line.checked_add_signed(row as i64);
        self.selection
            .as_ref()
            .zip(line)
            .is_some_and(|(selection, line)| selection.contains(line, col))
    }
}
