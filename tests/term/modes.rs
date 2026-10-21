use super::{cursor, feed, lines};
use trame::term::{CursorShape, Modes};

#[test]
fn alternate_screen_restores_primary_content_and_cursor() {
    let mut term = feed(3, 6, "keep\r\nme\x1b[?1049h");
    assert_eq!(lines(&term), vec!["", "", ""]);
    term.feed(b"vim stuff\x1b[?1049l");
    assert_eq!(lines(&term), vec!["keep", "me", ""]);
    assert_eq!(cursor(&term), (1, 2));
}

#[test]
fn alternate_screen_has_no_history() {
    let mut term = feed(2, 6, "\x1b[?1049h");
    term.feed(b"1\r\n2\r\n3\r\n4");
    assert_eq!(term.screen().grid().history_len(), 0);
}

#[test]
fn private_modes_toggle_flags() {
    let term = feed(2, 6, "\x1b[?1h\x1b[?2004h\x1b[?25l\x1b[?1002h\x1b[?1006h");
    let modes = term.screen().modes();
    assert!(modes.contains(Modes::APP_CURSOR));
    assert!(modes.contains(Modes::BRACKETED_PASTE));
    assert!(!modes.contains(Modes::CURSOR_VISIBLE));
    assert!(modes.contains(Modes::MOUSE_DRAG));
    assert!(modes.contains(Modes::MOUSE_SGR));
    let term = feed(2, 6, "\x1b[?1000h\x1b[?1003h");
    let modes = term.screen().modes();
    assert!(!modes.contains(Modes::MOUSE_CLICK));
    assert!(modes.contains(Modes::MOUSE_MOTION));
}

#[test]
fn autowrap_off_overwrites_last_column() {
    let term = feed(2, 3, "\x1b[?7labcdef");
    assert_eq!(lines(&term), vec!["abf", ""]);
}

#[test]
fn insert_mode_shifts_text() {
    let term = feed(1, 6, "abc\x1b[1;1H\x1b[4hX");
    assert_eq!(lines(&term)[0], "Xabc");
}

#[test]
fn origin_mode_positions_relative_to_region() {
    let term = feed(5, 5, "\x1b[2;4r\x1b[?6h\x1b[1;1Hx");
    assert_eq!(lines(&term)[1], "x");
}

#[test]
fn cursor_shape_follows_decscusr() {
    let term = feed(1, 4, "\x1b[5 q");
    assert_eq!(term.screen().cursor().shape, CursorShape::Bar);
}

#[test]
fn save_and_restore_cursor() {
    let term = feed(3, 6, "\x1b[2;3H\x1b7\x1b[1;1H\x1b8x");
    assert_eq!(lines(&term)[1], "  x");
}
