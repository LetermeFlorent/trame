use super::CellText;
use crate::term::Marks;
use crate::text::{DEFAULT_LINE_HEIGHT, MAX_CACHED_GLYPHS, TextSystem};

const COMBINING_MARKS: u32 = 112;

fn marked(first: u32, second: u32) -> CellText {
    let mark = |offset| char::from_u32(0x300 + offset).expect("a combining mark");
    CellText {
        ch: 'a',
        marks: Marks::default().with(mark(first)).with(mark(second)),
        bold: false,
        italic: false,
        columns: 1,
    }
}

#[test]
fn the_cell_cache_stays_under_its_bound() {
    let mut system =
        TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("a monospace font is installed");
    let texts = (0..COMBINING_MARKS)
        .flat_map(|first| (0..COMBINING_MARKS).map(move |second| marked(first, second)));
    for text in texts.take(MAX_CACHED_GLYPHS + 1) {
        system.cell_glyph(text);
        system.cluster_glyphs(text);
    }
    assert!(system.cells.glyphs.len() <= MAX_CACHED_GLYPHS);
    assert!(system.cells.clusters.len() <= MAX_CACHED_GLYPHS);
}
