mod control;
mod cursor_ops;
mod perform;
mod print;
mod resize;
mod view;

use super::charset::CharsetState;
use super::cursor::{Cursor, CursorShape, SavedCursor};
use super::grid::Grid;
use super::modes::Modes;

pub use view::{CursorView, ReportedColors};

const TAB_WIDTH: usize = 8;

#[derive(Debug, Clone)]
pub struct Screen {
    pub(crate) primary: Grid,
    pub(crate) alternate: Grid,
    pub(crate) cursor: Cursor,
    pub(crate) saved: [SavedCursor; 2],
    pub(crate) modes: Modes,
    pub(crate) charsets: CharsetState,
    pub(crate) scroll_top: usize,
    pub(crate) scroll_bottom: usize,
    pub(crate) tab_stops: Vec<bool>,
    pub(crate) cursor_shape: CursorShape,
    pub(crate) title: Option<String>,
    pub(crate) cwd: Option<String>,
    pub(crate) responses: Vec<u8>,
    pub(crate) last_char: Option<char>,
    pub(crate) bell: bool,
    pub(crate) clipboard: Option<String>,
    pub(crate) colors: ReportedColors,
}

impl Screen {
    pub fn new(rows: usize, cols: usize, max_history: usize) -> Self {
        let rows = rows.max(1);
        let cols = cols.max(1);
        Self {
            primary: Grid::new(rows, cols, max_history),
            alternate: Grid::new(rows, cols, 0),
            cursor: Cursor::default(),
            saved: [SavedCursor::default(); 2],
            modes: Modes::default(),
            charsets: CharsetState::default(),
            scroll_top: 0,
            scroll_bottom: rows - 1,
            tab_stops: default_tab_stops(cols),
            cursor_shape: CursorShape::default(),
            title: None,
            cwd: None,
            responses: Vec::new(),
            last_char: None,
            bell: false,
            clipboard: None,
            colors: ReportedColors::default(),
        }
    }

    pub(crate) fn alt_active(&self) -> bool {
        self.modes.contains(Modes::ALT_SCREEN)
    }

    pub(crate) fn grid_mut(&mut self) -> &mut Grid {
        if self.alt_active() {
            &mut self.alternate
        } else {
            &mut self.primary
        }
    }

    pub(crate) fn respond(&mut self, bytes: &[u8]) {
        self.responses.extend_from_slice(bytes);
    }

    pub(crate) fn full_reset(&mut self) {
        let (rows, cols) = (self.rows(), self.cols());
        let history = std::mem::replace(&mut self.primary, Grid::new(1, 1, 0));
        let mut fresh = Self::new(rows, cols, 0);
        fresh.primary = history;
        fresh.primary.reset();
        fresh.title = self.title.take();
        fresh.cwd = self.cwd.take();
        fresh.colors = self.colors;
        fresh.responses = std::mem::take(&mut self.responses);
        *self = fresh;
    }

    pub(crate) fn soft_reset(&mut self) {
        self.modes.set(Modes::CURSOR_VISIBLE, true);
        self.modes.set(Modes::ORIGIN, false);
        self.modes.set(Modes::AUTOWRAP, true);
        self.modes.set(Modes::INSERT, false);
        self.modes.set(Modes::APP_CURSOR, false);
        self.modes.set(Modes::APP_KEYPAD, false);
        self.scroll_top = 0;
        self.scroll_bottom = self.rows() - 1;
        self.charsets = CharsetState::default();
        self.cursor.template = Default::default();
        self.saved = [SavedCursor::default(); 2];
    }
}

pub(crate) fn default_tab_stops(cols: usize) -> Vec<bool> {
    (0..cols)
        .map(|col| col > 0 && col % TAB_WIDTH == 0)
        .collect()
}
