use trame::term::SelectionKind;

use super::{select, text};
use crate::feed;

#[test]
fn selection_stays_on_the_same_text_when_the_view_scrolls() {
    let mut term = feed(3, 6, "one\r\ntwo\r\nthree");
    select(&mut term, SelectionKind::Simple, (1, 0), (1, 2));
    assert_eq!(text(&term).as_deref(), Some("two"));
    term.feed(b"\r\nfour\r\nfive");
    assert_eq!(text(&term).as_deref(), Some("two"));
    term.screen_mut().scroll_display(2);
    let grid = term.screen().grid();
    let shown = (0..grid.rows()).find(|row| grid.is_selected_in_view(*row as isize, 0));
    assert_eq!(
        shown.map(|row| grid.visible_row(row).text()).as_deref(),
        Some("two")
    );
}

#[test]
fn selection_spans_history_and_screen() {
    let mut term = feed(2, 6, "one\r\ntwo\r\nthree\r\nfour");
    term.screen_mut().scroll_display(2);
    select(&mut term, SelectionKind::Simple, (0, 0), (1, 5));
    assert_eq!(text(&term).as_deref(), Some("one\ntwo"));
}

#[test]
fn rewriting_a_selected_line_clears_the_selection() {
    let mut term = feed(3, 10, "one\r\ntwo\r\nthree");
    select(&mut term, SelectionKind::Simple, (0, 0), (1, 2));
    term.feed(b"\x1b[3;1Hx");
    assert!(term.screen().selection().is_some());
    term.feed(b"\x1b[2;1Hx");
    assert!(term.screen().selection().is_none());
}

#[test]
fn erasing_a_selected_line_clears_the_selection() {
    let mut term = feed(3, 10, "one\r\ntwo\r\nthree");
    select(&mut term, SelectionKind::Simple, (0, 0), (0, 2));
    term.feed(b"\x1b[2J");
    assert!(term.screen().selection().is_none());
}

#[test]
fn scrolling_a_region_over_the_selection_clears_it() {
    let mut term = feed(4, 6, "a\r\nb\r\nc\r\nd");
    select(&mut term, SelectionKind::Simple, (1, 0), (1, 0));
    term.screen_mut().extend_selection(2, 0);
    term.feed(b"\x1b[2;3r\x1b[3;1H\n");
    assert!(term.screen().selection().is_none());
}

#[test]
fn clear_selection_and_resize_drop_it() {
    let mut term = feed(2, 10, "hello");
    select(&mut term, SelectionKind::Simple, (0, 0), (0, 3));
    term.screen_mut().clear_selection();
    assert!(term.screen().selection().is_none());
    select(&mut term, SelectionKind::Simple, (0, 0), (0, 3));
    term.screen_mut().resize(3, 8);
    assert!(term.screen().selection().is_none());
}
