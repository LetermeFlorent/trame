use trame::term::Terminal;

use super::fixture::put;

fn tiles(term: &Terminal) -> Vec<(isize, f32, f32)> {
    term.screen()
        .image_tiles()
        .iter()
        .map(|tile| (tile.row, tile.rows, tile.uv[1]))
        .collect()
}

#[test]
fn an_image_scrolled_off_the_alternate_screen_keeps_its_lower_rows() {
    let mut term = Terminal::new(6, 10, 10);
    term.feed(b"\x1b[?1049h\x1b[1;1H");
    put(&mut term, "i=1,c=2,r=4");
    term.feed(b"\x1b[6;1H\n");
    assert_eq!(tiles(&term), vec![(0, 3.0, 0.25)]);
    term.feed(b"\n\n");
    assert_eq!(tiles(&term), vec![(0, 1.0, 0.75)]);
    term.feed(b"\n");
    assert!(tiles(&term).is_empty());
}

#[test]
fn an_image_scrolled_out_of_a_region_keeps_its_lower_rows() {
    let mut term = Terminal::new(6, 10, 10);
    term.feed(b"\x1b[2;5r\x1b[2;1H");
    put(&mut term, "i=1,c=2,r=3");
    term.feed(b"\x1b[5;1H\n");
    assert_eq!(tiles(&term), vec![(1, 2.0, 1.0 / 3.0)]);
}

#[test]
fn an_image_scrolled_off_a_screen_without_history_keeps_its_lower_rows() {
    let mut term = Terminal::new(4, 10, 0);
    put(&mut term, "i=1,c=2,r=2");
    term.feed(b"\x1b[4;1H\n");
    assert_eq!(tiles(&term), vec![(0, 1.0, 0.5)]);
}
