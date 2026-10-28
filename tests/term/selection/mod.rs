mod lifetime;
mod words;

use trame::term::{SelectionKind, Terminal};

use super::{feed, terminal};

fn select(term: &mut Terminal, kind: SelectionKind, from: (usize, usize), to: (usize, usize)) {
    let screen = term.screen_mut();
    screen.start_selection(kind, from.0, from.1);
    screen.extend_selection(to.0, to.1);
}

fn text(term: &Terminal) -> Option<String> {
    let text = term.screen().selected_text();
    assert_eq!(term.screen().has_selected_text(), text.is_some());
    text
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
fn blank_rows_copy_as_empty_lines() {
    let mut term = terminal(2, 4);
    select(&mut term, SelectionKind::Line, (0, 0), (1, 3));
    assert_eq!(text(&term).as_deref(), Some("\n"));
}

#[test]
fn a_blank_selection_has_no_text() {
    let mut term = feed(2, 10, "ab");
    select(&mut term, SelectionKind::Simple, (0, 4), (0, 8));
    assert_eq!(text(&term), None);
}
