pub mod charset;

use crate::term::cell::Cell;
use charset::CharsetState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorShape {
    #[default]
    Block,
    Underline,
    Bar,
}

impl CursorShape {
    pub fn from_decscusr(value: u16) -> Self {
        match value {
            3 | 4 => Self::Underline,
            5 | 6 => Self::Bar,
            _ => Self::Block,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub template: Cell,
    pub pending_wrap: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SavedCursor {
    pub row: usize,
    pub col: usize,
    pub template: Cell,
    pub pending_wrap: bool,
    pub origin: bool,
    pub charsets: CharsetState,
}
