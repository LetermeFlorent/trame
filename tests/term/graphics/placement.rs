use trame::term::Terminal;
use trame::term::graphics::{Anchor, Crop};

use super::fixture::{all_placed, placed, put, send};
use crate::cursor;

#[test]
fn a_transmit_and_place_anchors_at_the_cursor_and_moves_past_the_image() {
    let mut term = Terminal::new(10, 20, 0);
    term.feed(b"\x1b[2;3H");
    put(&mut term, "i=1,c=4,r=3,z=-1");
    let anchor = &term.screen().grid().row(1).anchors()[0];
    let expected = Anchor {
        col: 2,
        image: 1,
        placement: 0,
        cols: 4,
        rows: 3,
        crop: Crop {
            x: 0,
            y: 0,
            w: 1,
            h: 1,
        },
        offset: (0, 0),
        z: -1,
    };
    assert_eq!(anchor, &expected);
    assert_eq!(cursor(&term), (3, 6));
}

#[test]
fn the_cursor_stays_with_c_1_and_stops_at_the_right_edge() {
    let mut term = Terminal::new(10, 8, 0);
    put(&mut term, "i=1,c=4,r=3,C=1");
    assert_eq!(cursor(&term), (0, 0));
    term.feed(b"\x1b[1;6H");
    put(&mut term, "i=1,c=4,r=1");
    assert_eq!(cursor(&term), (0, 7));
}

#[test]
fn an_image_taller_than_the_room_left_scrolls_the_screen() {
    let mut term = Terminal::new(4, 10, 10);
    term.feed(b"\x1b[3;1H");
    put(&mut term, "i=1,c=2,r=3");
    assert_eq!(cursor(&term), (3, 2));
    assert_eq!(placed(&term, 1), vec![(1, 0)]);
}

#[test]
fn placing_an_image_again_with_the_same_placement_id_moves_it() {
    let mut term = Terminal::new(6, 10, 0);
    put(&mut term, "i=1,p=7,c=1,r=1,C=1");
    term.feed(b"\x1b[5;1H");
    send(&mut term, "a=p,i=1,p=7,c=1,r=1,C=1");
    assert_eq!(
        term.screen_mut().take_responses(),
        b"\x1b_Gi=1,p=7;OK\x1b\\"
    );
    assert_eq!(all_placed(&term), vec![(1, 7)]);
    assert_eq!(placed(&term, 4), vec![(1, 7)]);
}

#[test]
fn unnamed_placements_add_up() {
    let mut term = Terminal::new(6, 10, 0);
    put(&mut term, "i=1,c=1,r=1,C=1");
    send(&mut term, "a=p,i=1,c=1,r=1,C=1");
    assert_eq!(placed(&term, 0), vec![(1, 0), (1, 0)]);
}

#[test]
fn placing_a_missing_image_fails() {
    let mut term = Terminal::new(6, 10, 0);
    send(&mut term, "a=p,i=5");
    assert_eq!(
        term.screen_mut().take_responses(),
        b"\x1b_Gi=5;ENOENT:no such image\x1b\\"
    );
    assert!(all_placed(&term).is_empty());
}

#[test]
fn a_virtual_placement_is_recorded_without_an_anchor() {
    let mut term = Terminal::new(6, 10, 0);
    put(&mut term, "i=3,U=1,c=5,r=2");
    assert_eq!(term.screen().graphics().virtuals.get(&3), Some(&(5, 2)));
    assert!(all_placed(&term).is_empty());
    assert_eq!(cursor(&term), (0, 0));
}

#[test]
fn the_cell_size_sets_the_default_span() {
    let mut term = Terminal::new(6, 10, 0);
    term.screen_mut().set_cell_pixels(1, 1);
    send(&mut term, "a=T,f=24,s=2,v=1,i=1,q=2;AAAAAAAA");
    let anchor = &term.screen().grid().row(0).anchors()[0];
    assert_eq!((anchor.cols, anchor.rows), (2, 1));
}

#[test]
fn retransmitting_an_image_removes_its_placements() {
    let mut term = Terminal::new(6, 10, 0);
    put(&mut term, "i=1,c=1,r=1,C=1");
    send(&mut term, "a=t,f=24,s=1,v=1,i=1,q=2;AAAA");
    assert!(all_placed(&term).is_empty());
    assert!(term.screen().graphics().get(1).is_some());
}
