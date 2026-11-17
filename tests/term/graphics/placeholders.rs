use trame::term::Terminal;
use trame::term::graphics::{ImageTile, Layer, PLACEHOLDER, diacritic_index};

use crate::cursor;

const PIXELS_4X2: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
const PIXELS_2X1: &str = "AAAAAAAAAAA=";

fn transmit(term: &mut Terminal, keys: &str, pixels: &str) {
    term.feed(format!("\x1b_Gf=32,q=2,{keys};{pixels}\x1b\\").as_bytes());
}

fn cells(marks: &str, count: usize) -> String {
    let rest: String = std::iter::repeat_n(PLACEHOLDER, count - 1).collect();
    format!("{PLACEHOLDER}{marks}{rest}")
}

fn shape(tile: &ImageTile) -> (isize, usize, f32, f32, [f32; 4], (u16, u16)) {
    (
        tile.row,
        tile.col,
        tile.cols,
        tile.rows,
        tile.uv,
        tile.offset,
    )
}

#[test]
fn the_diacritic_table_starts_and_ends_where_kitty_does() {
    assert_eq!(diacritic_index('\u{305}'), Some(0));
    assert_eq!(diacritic_index('\u{30D}'), Some(1));
    assert_eq!(diacritic_index('\u{1D244}'), Some(296));
    assert_eq!(diacritic_index('\u{300}'), None);
}

#[test]
fn a_placeholder_is_one_column_wide_and_keeps_its_diacritics() {
    let mut term = Terminal::new(2, 10, 0);
    term.feed("\u{10EEEE}\u{305}\u{30D}".as_bytes());
    let marks: Vec<char> = term.screen().grid().row(0).marks(0).chars().collect();
    assert_eq!(marks, vec!['\u{305}', '\u{30D}']);
    assert_eq!(cursor(&term), (0, 1));
}

#[test]
fn placeholder_rows_resolve_to_letterboxed_tiles_under_the_text() {
    let mut term = Terminal::new(4, 10, 0);
    transmit(&mut term, "a=T,U=1,i=7,c=4,r=2,s=4,v=2", PIXELS_4X2);
    let text = format!(
        "\x1b[38;5;7m{}\r\n{}",
        cells("\u{305}\u{305}", 4),
        cells("\u{30D}\u{305}", 4)
    );
    term.feed(text.as_bytes());
    let tiles = term.screen().image_tiles();
    assert_eq!(tiles.len(), 2);
    assert_eq!(
        shape(&tiles[0]),
        (0, 0, 4.0, 0.5, [0.0, 0.0, 1.0, 0.5], (0, 8))
    );
    assert_eq!(
        shape(&tiles[1]),
        (1, 0, 4.0, 0.5, [0.0, 0.5, 1.0, 1.0], (0, 0))
    );
    assert!(tiles.iter().all(|tile| tile.layer == Layer::UnderText));
}

#[test]
fn an_rgb_foreground_names_the_image_by_its_24_bits() {
    let mut term = Terminal::new(2, 10, 0);
    transmit(&mut term, "a=T,U=1,i=300,c=1,r=1,s=2,v=1", PIXELS_2X1);
    term.feed(format!("\x1b[38;2;0;1;44m{}", cells("\u{305}\u{305}", 1)).as_bytes());
    let tiles = term.screen().image_tiles();
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].image.width, 2);
}

#[test]
fn the_third_diacritic_sets_the_high_byte_of_the_id() {
    let mut term = Terminal::new(2, 10, 0);
    transmit(&mut term, "a=T,U=1,i=16777221,c=1,r=1,s=2,v=1", PIXELS_2X1);
    term.feed(format!("\x1b[38;5;5m{}", cells("\u{305}\u{305}", 1)).as_bytes());
    assert!(term.screen().image_tiles().is_empty());
    term.feed(format!("\r{}", cells("\u{305}\u{305}\u{30D}", 1)).as_bytes());
    assert_eq!(term.screen().image_tiles().len(), 1);
}

#[test]
fn a_placeholder_without_a_virtual_placement_draws_nothing() {
    let mut term = Terminal::new(2, 10, 0);
    transmit(&mut term, "a=t,i=9,s=2,v=1", PIXELS_2X1);
    term.feed(format!("\x1b[38;5;9m{}", cells("\u{305}\u{305}", 2)).as_bytes());
    assert!(term.screen().image_tiles().is_empty());
}

#[test]
fn a_placeholder_in_history_draws_when_scrolled_back_to() {
    let mut term = Terminal::new(2, 10, 10);
    transmit(&mut term, "a=T,U=1,i=7,c=2,r=1,s=2,v=1", PIXELS_2X1);
    term.feed(format!("\x1b[38;5;7m{}\r\n\n\n", cells("\u{305}\u{305}", 2)).as_bytes());
    assert!(term.screen().image_tiles().is_empty());
    term.screen_mut().scroll_display(2);
    let tiles = term.screen().image_tiles();
    assert_eq!(tiles.len(), 1);
    assert_eq!((tiles[0].row, tiles[0].col, tiles[0].cols), (0, 0, 2.0));
}

#[test]
fn rows_wholly_in_the_letterbox_margin_draw_nothing() {
    let mut term = Terminal::new(4, 10, 0);
    transmit(&mut term, "a=T,U=1,i=7,c=1,r=3,s=4,v=2", PIXELS_4X2);
    let text = format!(
        "\x1b[38;5;7m{}\r\n{}\r\n{}",
        cells("\u{305}", 1),
        cells("\u{30D}", 1),
        cells("\u{30E}", 1)
    );
    term.feed(text.as_bytes());
    let tiles = term.screen().image_tiles();
    assert_eq!(tiles.len(), 1);
    assert_eq!(
        shape(&tiles[0]),
        (1, 0, 1.0, 0.25, [0.0, 0.0, 1.0, 1.0], (0, 6))
    );
}
