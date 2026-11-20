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

#[test]
fn a_title_drops_the_status_marks_in_front_of_its_name() {
    let term = feed(1, 10, "\x1b]0;\u{2733} Claude Code\x07");
    assert_eq!(term.screen().title(), Some("Claude Code"));
    let term = feed(1, 10, "\x1b]0;\u{2810} Fix the build\x07");
    assert_eq!(term.screen().title(), Some("Fix the build"));
    let term = feed(1, 10, "\x1b]0;\u{e9}t\u{e9} ~/work\x07");
    assert_eq!(term.screen().title(), Some("\u{e9}t\u{e9} ~/work"));
    assert_eq!(feed(1, 10, "\x1b]0;\u{2733}\x07").screen().title(), None);
}
