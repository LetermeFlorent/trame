use std::ops::Range;

use super::{CursorPlace, drawn_rows};
use crate::term::Screen;
use crate::term::graphics::ImageTile;
use crate::term::grid::{Row, Selection};

pub struct ScreenCopy {
    pub(super) rows: Vec<(isize, Row)>,
    pub(super) cols: usize,
    pub(super) cursor: Option<CursorPlace>,
    pub tiles: Vec<ImageTile>,
    selection: Option<Selection>,
    first_line: u64,
}

impl ScreenCopy {
    pub fn of(screen: &Screen, scroll: f32) -> Self {
        let grid = screen.grid();
        let drawn = drawn_rows(screen, scroll);
        Self {
            tiles: tiles(screen, &drawn),
            rows: drawn
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

fn tiles(screen: &Screen, drawn: &Range<isize>) -> Vec<ImageTile> {
    let first = screen.grid().first_visible_line();
    let top = first.saturating_add_signed(drawn.start as i64);
    let shift = top as isize - first as isize;
    let rows = usize::try_from(drawn.end - shift).unwrap_or(0);
    let mut tiles = screen.image_tiles_at(top, rows);
    for tile in &mut tiles {
        tile.row += shift;
    }
    tiles
}
