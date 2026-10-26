use super::{cursor, feed, lines, terminal};

#[test]
fn linefeed_at_bottom_pushes_into_history() {
    let term = feed(2, 5, "one\r\ntwo\r\nthree");
    assert_eq!(lines(&term), vec!["two", "three"]);
    let grid = term.screen().grid();
    assert_eq!(grid.history_len(), 1);
    assert_eq!(grid.line(grid.first_line()).unwrap().text(), "one");
}

#[test]
fn scroll_region_keeps_outside_rows() {
    let term = feed(4, 5, "top\r\na\r\nb\r\nbot\x1b[2;3r\x1b[3;1H\nnew");
    assert_eq!(lines(&term), vec!["top", "b", "new", "bot"]);
    assert_eq!(term.screen().grid().history_len(), 0);
}

#[test]
fn reverse_index_at_top_scrolls_down() {
    let term = feed(3, 5, "a\r\nb\r\nc\x1b[1;1H\x1bM");
    assert_eq!(lines(&term), vec!["", "a", "b"]);
}

#[test]
fn history_is_capped() {
    let mut term = terminal(2, 4);
    for index in 0..300 {
        term.feed(format!("{index}\r\n").as_bytes());
    }
    assert_eq!(term.screen().grid().history_len(), 100);
}

#[test]
fn a_lower_history_limit_drops_only_the_oldest_lines() {
    let mut term = terminal(2, 4);
    for index in 0..50 {
        term.feed(format!("{index}\r\n").as_bytes());
    }
    term.screen_mut().set_history(10);
    let grid = term.screen().grid();
    assert_eq!(grid.history_len(), 10);
    assert_eq!(grid.line(grid.first_line()).unwrap().text(), "39");
    assert_eq!(lines(&term), vec!["49", ""]);
}

#[test]
fn display_offset_scrolls_back_and_stays_on_content() {
    let mut term = feed(2, 5, "1\r\n2\r\n3\r\n4");
    term.screen_mut().scroll_display(2);
    let grid = term.screen().grid();
    assert_eq!(grid.visible_row(0).text(), "1");
    term.feed(b"\r\n5");
    let grid = term.screen().grid();
    assert_eq!(grid.visible_row(0).text(), "1");
    assert_eq!(grid.display_offset(), 3);
}

#[test]
fn shrinking_rows_keeps_cursor_line_visible() {
    let mut term = feed(4, 5, "a\r\nb\r\nc\r\nd");
    term.screen_mut().resize(2, 5);
    assert_eq!(lines(&term), vec!["c", "d"]);
    assert_eq!(cursor(&term), (1, 1));
    assert_eq!(term.screen().grid().history_len(), 2);
}

#[test]
fn shrinking_rows_drops_blank_rows_below_cursor_first() {
    let mut term = feed(4, 5, "a");
    term.screen_mut().resize(2, 5);
    assert_eq!(lines(&term), vec!["a", ""]);
    assert_eq!(term.screen().grid().history_len(), 0);
}

#[test]
fn growing_rows_adds_blank_lines() {
    let mut term = feed(2, 5, "a\r\nb");
    term.screen_mut().resize(4, 8);
    assert_eq!(lines(&term), vec!["a", "b", "", ""]);
    assert_eq!(cursor(&term), (1, 1));
}

#[test]
fn erase_scrollback_clears_history() {
    let term = feed(2, 5, "1\r\n2\r\n3\x1b[3J");
    assert_eq!(term.screen().grid().history_len(), 0);
}
