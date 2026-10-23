use super::{cursor, feed, lines};

#[test]
fn wraps_at_last_column_only_when_next_char_arrives() {
    let term = feed(3, 4, "abcd");
    assert_eq!(lines(&term), vec!["abcd", "", ""]);
    assert_eq!(cursor(&term), (0, 3));
    let term = feed(3, 4, "abcde");
    assert_eq!(lines(&term), vec!["abcd", "e", ""]);
    assert!(term.screen().grid().row(0).wrapped);
}

#[test]
fn carriage_return_cancels_pending_wrap() {
    let term = feed(3, 4, "abcd\rX");
    assert_eq!(lines(&term), vec!["Xbcd", "", ""]);
}

#[test]
fn moves_cursor_with_cup_and_clamps() {
    let term = feed(5, 10, "\x1b[3;4Hx\x1b[99;99Hy");
    assert_eq!(lines(&term)[2], "   x");
    assert_eq!(lines(&term)[4], "         y");
}

#[test]
fn erases_display_and_line() {
    let term = feed(3, 5, "aaaaa\r\nbbbbb\r\nccccc\x1b[2;3H\x1b[0J");
    assert_eq!(lines(&term), vec!["aaaaa", "bb", ""]);
    let term = feed(2, 5, "abcde\x1b[1;3H\x1b[1K");
    assert_eq!(lines(&term)[0], "   de");
    let term = feed(2, 5, "abcde\x1b[2K");
    assert_eq!(lines(&term)[0], "");
}

#[test]
fn inserts_and_deletes_characters() {
    let term = feed(1, 6, "abcdef\x1b[1;2H\x1b[2@");
    assert_eq!(lines(&term)[0], "a  bcd");
    let term = feed(1, 6, "abcdef\x1b[1;2H\x1b[2P");
    assert_eq!(lines(&term)[0], "adef");
    let term = feed(1, 6, "abcdef\x1b[1;2H\x1b[3X");
    assert_eq!(lines(&term)[0], "a   ef");
}

#[test]
fn inserts_and_deletes_lines_inside_region() {
    let term = feed(4, 3, "a\r\nb\r\nc\r\nd\x1b[2;1H\x1b[L");
    assert_eq!(lines(&term), vec!["a", "", "b", "c"]);
    let term = feed(4, 3, "a\r\nb\r\nc\r\nd\x1b[2;1H\x1b[M");
    assert_eq!(lines(&term), vec!["a", "c", "d", ""]);
    assert_eq!(term.screen().grid().history_len(), 0);
}

#[test]
fn wide_characters_take_two_cells_and_wrap_early() {
    let term = feed(2, 3, "a漢字");
    assert_eq!(lines(&term), vec!["a漢", "字"]);
    assert_eq!(cursor(&term), (1, 2));
}

#[test]
fn overwriting_half_of_wide_char_clears_the_other_half() {
    let term = feed(1, 4, "漢\x1b[1;2Hx");
    assert_eq!(lines(&term)[0], " x");
}

#[test]
fn tab_moves_to_next_stop() {
    let term = feed(1, 20, "a\tb");
    assert_eq!(lines(&term)[0], "a       b");
}

#[test]
fn line_drawing_charset_maps_letters() {
    let term = feed(1, 5, "\x1b(0lqk\x1b(Bq");
    assert_eq!(lines(&term)[0], "┌─┐q");
}

#[test]
fn repeat_prints_last_char() {
    let term = feed(1, 8, "x\x1b[3b");
    assert_eq!(lines(&term)[0], "xxxx");
}

#[test]
fn the_cursor_line_joins_the_rows_it_wrapped_over() {
    let term = feed(4, 6, "ok\r\nabcdefghij");
    assert_eq!(term.screen().cursor_line(), "abcdefghij");
    let term = feed(4, 6, "abcdefgh\r\nnext");
    assert_eq!(term.screen().cursor_line(), "next");
    let term = feed(2, 6, "");
    assert_eq!(term.screen().cursor_line(), "");
}
