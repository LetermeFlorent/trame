use crate::feed;
use trame::term::{Color, Flags};

#[test]
fn sgr_sets_basic_bright_and_extended_colors() {
    let term = feed(
        1,
        10,
        "\x1b[31;102ma\x1b[38;5;200mb\x1b[38;2;1;2;3mc\x1b[48:2::9:8:7md",
    );
    let row = term.screen().grid().row(0);
    assert_eq!(row.cell(0).fg, Color::Indexed(1));
    assert_eq!(row.cell(0).bg, Color::Indexed(10));
    assert_eq!(row.cell(1).fg, Color::Indexed(200));
    assert_eq!(row.cell(2).fg, Color::Rgb(1, 2, 3));
    assert_eq!(row.cell(3).bg, Color::Rgb(9, 8, 7));
}

#[test]
fn sgr_flags_toggle_and_reset() {
    let term = feed(1, 10, "\x1b[1;3;4ma\x1b[22;23mb\x1b[4:3mc\x1b[0md");
    let row = term.screen().grid().row(0);
    assert!(row.cell(0).flags.contains(Flags::BOLD));
    assert!(row.cell(0).flags.contains(Flags::ITALIC));
    assert!(row.cell(0).flags.contains(Flags::UNDERLINE));
    assert!(!row.cell(1).flags.contains(Flags::BOLD));
    assert!(row.cell(1).flags.contains(Flags::UNDERLINE));
    assert!(row.cell(2).flags.contains(Flags::CURLY_UNDERLINE));
    assert!(!row.cell(2).flags.contains(Flags::UNDERLINE));
    assert_eq!(row.cell(3).flags, Flags::empty());
    assert_eq!(row.cell(3).fg, Color::Foreground);
}

#[test]
fn erase_uses_current_background() {
    let term = feed(2, 4, "\x1b[44m\x1b[2J");
    let row = term.screen().grid().row(1);
    assert_eq!(row.cell(3).bg, Color::Indexed(4));
}

#[test]
fn osc_sets_title_and_cwd() {
    let term = feed(
        1,
        10,
        "\x1b]0;mon titre\x07\x1b]7;file://localhost/C:/Users/Me%20Doc\x1b\\",
    );
    assert_eq!(term.screen().title(), Some("mon titre"));
    assert_eq!(term.screen().cwd(), Some("C:/Users/Me Doc"));
    let term = feed(1, 10, "\x1b]7;file:///home/me\x07");
    assert_eq!(term.screen().cwd(), Some("/home/me"));
    let term = feed(1, 10, "\x1b]9;9;\"D:\\work\"\x07");
    assert_eq!(term.screen().cwd(), Some("D:\\work"));
}

#[test]
fn a_title_drops_direction_controls() {
    let term = feed(1, 10, "\x1b]2;\u{202E}fdp.exe\u{2066}\u{200B}\u{E0041}\x07");
    assert_eq!(term.screen().title(), Some("fdp.exe"));
    let term = feed(1, 10, "\x1b]2;a\u{200D}b\x07");
    assert_eq!(term.screen().title(), Some("a\u{200D}b"));
}

#[test]
fn osc_52_stores_clipboard_text() {
    let mut term = feed(1, 10, "\x1b]52;c;aGVsbG8gd29ybGQ=\x07");
    assert_eq!(
        term.screen_mut().take_clipboard().as_deref(),
        Some("hello world")
    );
}
