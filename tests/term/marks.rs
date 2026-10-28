use trame::term::SelectionKind;

use super::{cursor, feed, lines};

#[test]
fn a_combining_mark_stays_on_its_base_cell() {
    let term = feed(1, 10, "e\u{301}x");
    assert_eq!(lines(&term), vec!["e\u{301}x"]);
    assert_eq!(cursor(&term), (0, 2));
}

#[test]
fn joiners_and_variation_selectors_are_kept() {
    let text = "\u{1F468}\u{200D}\u{1F469} \u{2764}\u{FE0F}";
    let term = feed(1, 10, text);
    assert_eq!(lines(&term), vec![text]);
    assert_eq!(cursor(&term), (0, 6));
}

#[test]
fn a_mark_after_a_wide_character_joins_its_first_cell() {
    let term = feed(1, 10, "\u{1F600}\u{FE0F}x");
    assert_eq!(lines(&term), vec!["\u{1F600}\u{FE0F}x"]);
}

#[test]
fn a_mark_at_the_last_column_joins_the_last_cell() {
    let term = feed(1, 3, "abc\u{301}");
    assert_eq!(lines(&term), vec!["abc\u{301}"]);
}

#[test]
fn a_cell_keeps_three_marks() {
    let term = feed(1, 10, "a\u{300}\u{301}\u{302}\u{303}");
    assert_eq!(lines(&term), vec!["a\u{300}\u{301}\u{302}"]);
}

#[test]
fn writing_over_a_cell_drops_its_marks() {
    let term = feed(1, 10, "e\u{301}\rx");
    assert_eq!(lines(&term), vec!["x"]);
    let term = feed(1, 10, "\u{1F600}\u{FE0F}\x1b[2Gx");
    assert_eq!(lines(&term), vec![" x"]);
}

#[test]
fn copying_keeps_the_marks() {
    let mut term = feed(1, 10, "e\u{301}x");
    let screen = term.screen_mut();
    screen.start_selection(SelectionKind::Simple, 0, 0);
    screen.extend_selection(0, 1);
    assert_eq!(screen.selected_text().as_deref(), Some("e\u{301}x"));
}
