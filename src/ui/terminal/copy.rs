use std::sync::Arc;

use super::{CursorPlace, drawn_rows};
use crate::term::grid::{Row, Selection};
use crate::term::{Screen, StoredImage};

pub struct ScreenCopy {
    pub(super) rows: Vec<(isize, Row)>,
    pub(super) cols: usize,
    pub(super) cursor: Option<CursorPlace>,
    selection: Option<Selection>,
    first_line: u64,
    images: Vec<(u32, Arc<StoredImage>)>,
    session_id: u64,
}

impl ScreenCopy {
    pub fn of(screen: &Screen, scroll: f32) -> Self {
        let grid = screen.grid();
        let rows: Vec<(isize, Row)> = drawn_rows(screen, scroll)
            .filter_map(|row| Some((row, grid.view_row(row)?.clone())))
            .collect();
        let mut images: Vec<(u32, Arc<StoredImage>)> = Vec::new();
        for (_, row) in &rows {
            for id in row.cells().iter().filter_map(|cell| cell.image_id()) {
                if !images.iter().any(|(known, _)| *known == id)
                    && let Some(stored) = screen.image(id)
                {
                    images.push((id, stored.clone()));
                }
            }
        }
        Self {
            rows,
            cols: grid.cols(),
            cursor: CursorPlace::locate(screen),
            selection: grid.selection().cloned(),
            first_line: grid.first_visible_line(),
            images,
            session_id: 0,
        }
    }

    pub fn with_session(mut self, session_id: u64) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    pub fn image(&self, id: u32) -> Option<&Arc<StoredImage>> {
        self.images
            .iter()
            .find(|(known, _)| *known == id)
            .map(|(_, image)| image)
    }

    pub fn is_selected(&self, row: isize, col: usize) -> bool {
        let line = self.first_line.checked_add_signed(row as i64);
        self.selection
            .as_ref()
            .zip(line)
            .is_some_and(|(selection, line)| selection.contains(line, col))
    }
}
