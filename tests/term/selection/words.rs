use trame::term::SelectionKind;

use super::text;
use crate::feed;

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
