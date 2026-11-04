use crate::feed;

#[test]
fn osc_sets_the_title() {
    let term = feed(1, 10, "\x1b]0;mon titre\x07");
    assert_eq!(term.screen().title(), Some("mon titre"));
}

#[test]
fn a_title_drops_direction_controls() {
    let term = feed(1, 10, "\x1b]2;\u{202E}fdp.exe\u{2066}\u{200B}\u{E0041}\x07");
    assert_eq!(term.screen().title(), Some("fdp.exe"));
    let term = feed(1, 10, "\x1b]2;a\u{200D}b\x07");
    assert_eq!(term.screen().title(), Some("a\u{200D}b"));
}
