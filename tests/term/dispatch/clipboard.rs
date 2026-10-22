use crate::feed;

#[test]
fn osc_52_accepts_the_string_terminator() {
    let mut term = feed(1, 10, "\x1b]52;c;aGk=\x1b\\");
    assert_eq!(term.screen_mut().take_clipboard().as_deref(), Some("hi"));
}

#[test]
fn osc_52_works_for_other_selection_names() {
    let mut term = feed(1, 10, "\x1b]52;p;aGk=\x07");
    assert_eq!(term.screen_mut().take_clipboard().as_deref(), Some("hi"));
    let mut term = feed(1, 10, "\x1b]52;;aGk=\x07");
    assert_eq!(term.screen_mut().take_clipboard().as_deref(), Some("hi"));
}

#[test]
fn osc_52_keeps_text_without_padding_and_with_utf8() {
    let mut term = feed(1, 10, "\x1b]52;c;aMOpbGxv\x07");
    assert_eq!(
        term.screen_mut().take_clipboard().as_deref(),
        Some("h\u{e9}llo")
    );
    let mut term = feed(1, 10, "\x1b]52;c;YWI\x07");
    assert_eq!(term.screen_mut().take_clipboard().as_deref(), Some("ab"));
}

#[test]
fn osc_52_query_is_ignored() {
    let mut term = feed(1, 10, "\x1b]52;c;?\x07");
    assert_eq!(term.screen_mut().take_clipboard(), None);
    assert!(term.screen_mut().take_responses().is_empty());
}

#[test]
fn osc_52_ignores_invalid_base64() {
    let mut term = feed(1, 10, "\x1b]52;c;@@@@\x07");
    assert_eq!(term.screen_mut().take_clipboard(), None);
}

#[test]
fn osc_52_invalid_payload_keeps_the_pending_text() {
    let mut term = feed(1, 10, "\x1b]52;c;aGk=\x07\x1b]52;c;@@@@\x07");
    assert_eq!(term.screen_mut().take_clipboard().as_deref(), Some("hi"));
}

#[test]
fn osc_52_latest_text_wins_and_is_taken_once() {
    let mut term = feed(1, 10, "\x1b]52;c;b25l\x07\x1b]52;c;dHdv\x07");
    assert_eq!(term.screen_mut().take_clipboard().as_deref(), Some("two"));
    assert_eq!(term.screen_mut().take_clipboard(), None);
}

#[test]
fn osc_52_without_data_does_nothing() {
    let mut term = feed(1, 10, "\x1b]52;c\x07");
    assert_eq!(term.screen_mut().take_clipboard(), None);
}
