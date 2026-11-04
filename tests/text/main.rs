mod atlas;
mod cells;
mod images;
mod layout;
mod marks;
#[cfg(windows)]
mod utf16;

use trame::term::Marks;
use trame::text::{CellText, DEFAULT_LINE_HEIGHT, TextStyle, TextSystem};

const SIZE: f32 = 14.0;

fn system() -> TextSystem {
    TextSystem::new(SIZE, DEFAULT_LINE_HEIGHT).expect("a monospace font is installed")
}

fn interface(weight: u16) -> TextStyle {
    TextStyle { size: 12.5, weight }
}

fn cell(ch: char, bold: bool, italic: bool, columns: u8) -> CellText {
    CellText {
        ch,
        marks: Marks::default(),
        bold,
        italic,
        columns,
    }
}

fn plain(ch: char) -> CellText {
    cell(ch, false, false, 1)
}
