pub mod cell;
pub mod cursor;
mod dispatch;
pub mod grid;
pub mod modes;
pub mod screen;
pub mod vt;

use vt::Parser;

pub use cell::{Cell, Color, Flags};
pub use cursor::CursorShape;
pub use grid::{GridPoint, Selection, SelectionKind};
pub use modes::Modes;
pub use screen::{CursorView, ReportedColors, Screen};

pub const DEFAULT_HISTORY: usize = 10_000;

#[derive(Debug)]
pub struct Terminal {
    parser: Parser,
    screen: Screen,
}

impl Terminal {
    pub fn new(rows: usize, cols: usize, max_history: usize) -> Self {
        Self {
            parser: Parser::new(),
            screen: Screen::new(rows, cols, max_history),
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.screen, bytes);
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }
}
