use trame::term::Terminal;
use trame::term::graphics::{ImageStore, Layer};

use super::fixture::{all_placed, placed, put, send};

#[test]
fn an_anchor_scrolls_into_history_and_still_draws_while_it_shows() {
    let mut term = Terminal::new(4, 10, 100);
    put(&mut term, "i=1,c=2,r=3,C=1");
    term.feed(b"\x1b[4;1H\n\n");
    let tiles = term.screen().image_tiles();
    assert_eq!(tiles.len(), 1);
    assert_eq!((tiles[0].row, tiles[0].col, tiles[0].rows), (-2, 0, 3.0));
    assert_eq!(tiles[0].uv, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(tiles[0].layer, Layer::OverText);
    term.feed(b"\n");
    assert!(term.screen().image_tiles().is_empty());
    let history = term.screen().grid().view_row(-3).unwrap();
    assert_eq!(history.anchors().len(), 1);
}

#[test]
fn scrolling_back_shows_the_anchor_at_its_view_row() {
    let mut term = Terminal::new(4, 10, 100);
    term.feed(b"\x1b[2;1H");
    put(&mut term, "i=1,c=2,r=1,C=1");
    term.feed(b"\x1b[4;1H\n\n\n\n\n");
    term.screen_mut().scroll_display(5);
    let tiles = term.screen().image_tiles();
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].row, 1);
}

#[test]
fn an_anchor_follows_a_scroll_region() {
    let mut term = Terminal::new(6, 10, 100);
    term.feed(b"\x1b[2;4r\x1b[3;1H");
    put(&mut term, "i=1,c=2,r=1,C=1");
    term.feed(b"\x1b[4;1H\n");
    assert_eq!(placed(&term, 1), vec![(1, 0)]);
    assert!(placed(&term, 2).is_empty());
}

#[test]
fn erasing_lines_and_printing_keep_the_anchor() {
    let mut term = Terminal::new(4, 10, 0);
    put(&mut term, "i=1,c=2,r=1,C=1");
    term.feed(b"xxxx\x1b[1;1H\x1b[K\x1b[J\x1b[2X");
    assert_eq!(placed(&term, 0), vec![(1, 0)]);
}

#[test]
fn clearing_the_screen_removes_anchors_but_keeps_the_image() {
    let mut term = Terminal::new(4, 10, 0);
    put(&mut term, "i=1,c=2,r=1,C=1");
    term.feed(b"\x1b[2J");
    assert!(all_placed(&term).is_empty());
    assert!(term.screen().graphics().get(1).is_some());
}

#[test]
fn clearing_the_history_frees_images_only_it_showed() {
    let mut term = Terminal::new(4, 10, 100);
    put(&mut term, "i=1,c=2,r=1,C=1");
    send(&mut term, "a=t,f=24,s=1,v=1,i=2,q=2;AAAA");
    term.feed(b"\x1b[4;1H\n\n\n\n\x1b[3J");
    assert!(term.screen().graphics().get(1).is_none());
    assert!(term.screen().graphics().get(2).is_some());
}

#[test]
fn the_alternate_screen_has_its_own_anchors() {
    let mut term = Terminal::new(4, 10, 0);
    put(&mut term, "i=1,c=2,r=1,C=1");
    term.feed(b"\x1b[?1049h");
    assert!(all_placed(&term).is_empty());
    put(&mut term, "i=2,c=2,r=1,C=1");
    assert_eq!(term.screen().image_tiles()[0].image.width, 1);
    assert_eq!(all_placed(&term), vec![(2, 0)]);
    term.feed(b"\x1b[?1049l");
    assert_eq!(all_placed(&term), vec![(1, 0)]);
    term.feed(b"\x1b[?1049h");
    assert!(all_placed(&term).is_empty());
}

#[test]
fn a_full_reset_clears_anchors_and_virtual_placements() {
    let mut term = Terminal::new(4, 10, 0);
    put(&mut term, "i=1,c=2,r=1,C=1");
    put(&mut term, "i=2,U=1,c=2,r=1");
    term.feed(b"\x1bc");
    assert!(all_placed(&term).is_empty());
    assert!(term.screen().graphics().virtuals.is_empty());
    assert!(term.screen().graphics().get(1).is_some());
}

#[test]
fn quota_eviction_prefers_unreferenced_images() {
    let mut term = Terminal::new(4, 10, 0);
    put(&mut term, "i=1,c=2,r=1,C=1");
    send(&mut term, "a=t,f=24,s=1,v=1,i=2,q=2;AAAA");
    let screen = term.screen();
    let mut store: ImageStore = screen.graphics().clone();
    let one = 4 + ImageStore::OVERHEAD;
    store.evict_unreferenced(one, [screen.grid(), screen.grid()]);
    assert!(store.get(1).is_some());
    assert!(store.get(2).is_none());
}

#[test]
fn layers_follow_the_z_index() {
    assert_eq!(Layer::of(i32::MIN), Layer::UnderBackground);
    assert_eq!(Layer::of(-1), Layer::UnderText);
    assert_eq!(Layer::of(0), Layer::OverText);
}
