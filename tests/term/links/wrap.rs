use trame::ui::terminal::links::{link_at, view_links};

use crate::feed;

const WRAPPED: &str = "go https://example.com/long/path end";

#[test]
fn a_link_that_wraps_continues_on_the_next_rows() {
    let term = feed(5, 10, WRAPPED);
    let url = "https://example.com/long/path".to_owned();
    let spans = view_links(term.screen(), 0..5);
    let places: Vec<_> = spans.iter().map(|s| (s.row, s.start, s.end)).collect();
    assert_eq!(places, [(0, 3, 10), (1, 0, 10), (2, 0, 10), (3, 0, 2)]);
    assert!(spans.iter().all(|span| span.target == url));
    for (row, col) in [(0, 3), (2, 5), (3, 1)] {
        assert_eq!(link_at(term.screen(), row, col), Some(url.clone()));
    }
    assert_eq!(link_at(term.screen(), 3, 4), None);
}

#[test]
fn only_the_drawn_rows_of_a_wrapped_link_are_returned() {
    let term = feed(5, 10, WRAPPED);
    let spans = view_links(term.screen(), 2..3);
    let rows: Vec<isize> = spans.iter().map(|span| span.row).collect();
    assert_eq!(rows, [2]);
}

#[test]
fn a_line_break_ends_the_link() {
    let term = feed(3, 20, "https://a.example/x\r\nmore/words");
    let spans = view_links(term.screen(), 0..3);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].row, 0);
}

#[test]
fn rows_above_the_history_do_not_hide_the_links_below() {
    let term = feed(3, 20, "x\r\nsee https://a.io");
    let spans = view_links(term.screen(), -2..3);
    assert_eq!(spans.len(), 1);
    assert_eq!((spans[0].row, spans[0].start, spans[0].end), (1, 4, 16));
}
