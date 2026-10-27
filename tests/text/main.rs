mod images;

use trame::text::{BOLD, CellText, DEFAULT_LINE_HEIGHT, FontRole, REGULAR, TextStyle, TextSystem};

const SIZE: f32 = 14.0;

fn system() -> TextSystem {
    TextSystem::new(SIZE, DEFAULT_LINE_HEIGHT).expect("a monospace font is installed")
}

fn interface(weight: u16) -> TextStyle {
    TextStyle {
        role: FontRole::Interface,
        size: 12.5,
        weight,
    }
}

fn cell(ch: char, bold: bool, italic: bool, columns: u8) -> CellText {
    CellText {
        ch,
        bold,
        italic,
        columns,
    }
}

#[test]
fn cell_metrics_fit_the_glyphs() {
    let metrics = system().cell_metrics();
    assert!(metrics.width >= 1.0 && metrics.width == metrics.width.round());
    assert!(metrics.height > metrics.width);
    assert!(metrics.baseline > 0.0 && metrics.baseline < metrics.height);
    assert!(metrics.underline_position > metrics.baseline);
    assert!(metrics.underline_position + metrics.underline_thickness <= metrics.height);
    assert!(metrics.strike_position < metrics.baseline);
}

#[test]
fn line_height_below_one_keeps_the_natural_height() {
    let mut text = system();
    let natural = text.cell_metrics().height;
    text.set_terminal_font(SIZE, 0.5);
    assert!(text.cell_metrics().height <= natural);
    assert!(text.cell_metrics().height >= SIZE);
}

#[test]
fn larger_fonts_give_larger_cells() {
    let mut text = system();
    let small = text.cell_metrics();
    text.set_terminal_font(SIZE * 2.0, DEFAULT_LINE_HEIGHT);
    let large = text.cell_metrics();
    assert!(large.width > small.width && large.height > small.height);
}

#[test]
fn blank_cells_have_no_glyph() {
    let mut text = system();
    assert!(text.cell_glyph(cell(' ', false, false, 1)).is_none());
    assert!(text.cell_glyph(cell('a', false, false, 1)).is_some());
}

#[test]
fn bold_and_italic_draw_different_glyphs() {
    let mut text = system();
    let regular = text.cell_glyph(cell('a', false, false, 1)).unwrap();
    let bold = text.cell_glyph(cell('a', true, false, 1)).unwrap();
    let italic = text.cell_glyph(cell('a', false, true, 1)).unwrap();
    assert_ne!(
        (regular.region.x, regular.region.y),
        (bold.region.x, bold.region.y)
    );
    assert_ne!(
        (regular.region.x, regular.region.y),
        (italic.region.x, italic.region.y)
    );
}

#[test]
fn repeated_cells_reuse_the_atlas() {
    let mut text = system();
    let first = text.cell_glyph(cell('x', false, false, 1)).unwrap();
    let second = text.cell_glyph(cell('x', false, false, 1)).unwrap();
    assert_eq!(
        (first.region.x, first.region.y),
        (second.region.x, second.region.y)
    );
    assert!(!text.end_frame());
}

#[cfg(any(windows, target_os = "macos"))]
#[test]
fn wide_characters_fit_two_cells() {
    let mut text = system();
    let width = text.cell_metrics().width;
    for ch in ['日', '한', '😀'] {
        let glyph = text
            .cell_glyph(cell(ch, false, false, 2))
            .expect("a fallback font covers it");
        assert!(glyph.x >= -1.0, "{ch} starts before its cell");
        assert!(
            glyph.x + glyph.region.width as f32 <= width * 2.0 + 1.0,
            "{ch} overflows two cells"
        );
    }
}

#[test]
fn empty_text_has_no_width() {
    let layout = system().layout("", interface(REGULAR));
    assert!(layout.glyphs.is_empty());
    assert_eq!(layout.width, 0.0);
}

#[test]
fn layout_width_grows_with_the_text() {
    let mut text = system();
    let short = text.layout("trame", interface(REGULAR));
    let long = text.layout("trame terminal", interface(REGULAR));
    assert!(long.width > short.width);
    assert!(short.cap_height > 0.0 && short.ascent > short.cap_height);
}

#[test]
fn layouts_are_cached_per_style() {
    let mut text = system();
    let regular = text.layout("Instances", interface(REGULAR));
    let again = text.layout("Instances", interface(REGULAR));
    let bold = text.layout("Instances", interface(BOLD));
    assert!(std::sync::Arc::ptr_eq(&regular, &again));
    assert!(!std::sync::Arc::ptr_eq(&regular, &bold));
    text.end_frame();
    let next_frame = text.layout("Instances", interface(REGULAR));
    assert!(std::sync::Arc::ptr_eq(&regular, &next_frame));
}
