use trame::term::{SelectionKind, Terminal};

use super::{feed, terminal};

fn select(term: &mut Terminal, kind: SelectionKind, from: (usize, usize), to: (usize, usize)) {
    let screen = term.screen_mut();
    screen.start_selection(kind, from.0, from.1);
    screen.extend_selection(to.0, to.1);
}

fn text(term: &Terminal) -> Option<String> {
    term.screen().selected_text()
}

#[test]
fn a_plain_click_selects_nothing() {
    let mut term = feed(2, 10, "hello");
    term.screen_mut()
        .start_selection(SelectionKind::Simple, 0, 2);
    assert_eq!(text(&term), None);
    assert!(!term.screen().grid().is_selected_in_view(0, 2));
}

#[test]
fn drag_selects_cells_inclusively() {
    let mut term = feed(2, 10, "hello world");
    select(&mut term, SelectionKind::Simple, (0, 1), (0, 3));
    assert_eq!(text(&term).as_deref(), Some("ell"));
    assert!(term.screen().grid().is_selected_in_view(0, 1));
    assert!(term.screen().grid().is_selected_in_view(0, 3));
    assert!(!term.screen().grid().is_selected_in_view(0, 4));
}

#[test]
fn dragging_backwards_selects_the_same_cells() {
    let mut term = feed(2, 10, "hello world");
    select(&mut term, SelectionKind::Simple, (0, 3), (0, 1));
    assert_eq!(text(&term).as_deref(), Some("ell"));
}

#[test]
fn lines_are_joined_with_newlines_and_trailing_spaces_trimmed() {
    let mut term = feed(3, 10, "ab  \r\n\r\ncd");
    select(&mut term, SelectionKind::Simple, (0, 0), (2, 1));
    assert_eq!(text(&term).as_deref(), Some("ab\n\ncd"));
}

#[test]
fn soft_wrapped_lines_join_without_a_newline() {
    let mut term = feed(3, 5, "abcdefgh\r\nij");
    select(&mut term, SelectionKind::Simple, (0, 0), (2, 1));
    assert_eq!(text(&term).as_deref(), Some("abcdefgh\nij"));
}

#[test]
fn a_wrap_keeps_the_spaces_at_the_break() {
    let mut term = feed(2, 5, "ab   cd");
    select(&mut term, SelectionKind::Simple, (0, 0), (1, 1));
    assert_eq!(text(&term).as_deref(), Some("ab   cd"));
}

#[test]
fn double_click_selects_a_word() {
    let mut term = feed(2, 20, "say hello_world now");
    term.screen_mut().start_selection(SelectionKind::Word, 0, 6);
    assert_eq!(text(&term).as_deref(), Some("hello_world"));
}

#[test]
fn word_stops_at_separators() {
    let mut term = feed(2, 30, "run (some-path/file.rs) now");
    term.screen_mut().start_selection(SelectionKind::Word, 0, 8);
    assert_eq!(text(&term).as_deref(), Some("some-path/file.rs"));
}

#[test]
fn double_click_on_blanks_selects_the_blank_run() {
    let mut term = feed(1, 12, "ab    cd");
    term.screen_mut().start_selection(SelectionKind::Word, 0, 3);
    assert!(term.screen().grid().is_selected_in_view(0, 2));
    assert!(term.screen().grid().is_selected_in_view(0, 5));
    assert!(!term.screen().grid().is_selected_in_view(0, 1));
}

#[test]
fn dragging_after_a_double_click_extends_by_words() {
    let mut term = feed(1, 30, "alpha beta gamma delta");
    term.screen_mut().start_selection(SelectionKind::Word, 0, 7);
    term.screen_mut().extend_selection(0, 13);
    assert_eq!(text(&term).as_deref(), Some("beta gamma"));
}

#[test]
fn triple_click_selects_the_whole_line() {
    let mut term = feed(3, 12, "first line\r\nsecond");
    term.screen_mut().start_selection(SelectionKind::Line, 0, 4);
    assert_eq!(text(&term).as_deref(), Some("first line"));
}

#[test]
fn triple_click_follows_a_wrapped_line() {
    let mut term = feed(3, 5, "abcdefgh\r\nij");
    term.screen_mut().start_selection(SelectionKind::Line, 1, 1);
    assert_eq!(text(&term).as_deref(), Some("abcdefgh"));
}

#[test]
fn extending_keeps_the_anchor() {
    let mut term = feed(2, 10, "0123456789");
    select(&mut term, SelectionKind::Simple, (0, 4), (0, 8));
    term.screen_mut().extend_selection(0, 1);
    assert_eq!(text(&term).as_deref(), Some("1234"));
}

#[test]
fn wide_characters_select_as_one_cell() {
    let mut term = feed(1, 10, "a\u{4e16}\u{754c}b");
    term.screen_mut()
        .start_selection(SelectionKind::Simple, 0, 2);
    term.screen_mut().extend_selection(0, 3);
    assert_eq!(text(&term).as_deref(), Some("\u{4e16}\u{754c}"));
}

#[test]
fn points_outside_the_grid_are_clamped() {
    let mut term = feed(2, 5, "abcde\r\nfghij");
    term.screen_mut()
        .start_selection(SelectionKind::Simple, 9, 99);
    term.screen_mut().extend_selection(0, 0);
    assert_eq!(text(&term).as_deref(), Some("abcde\nfghij"));
}

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

#[test]
fn blank_rows_copy_as_empty_lines() {
    let mut term = terminal(2, 4);
    select(&mut term, SelectionKind::Line, (0, 0), (1, 3));
    assert_eq!(text(&term).as_deref(), Some("\n"));
}
