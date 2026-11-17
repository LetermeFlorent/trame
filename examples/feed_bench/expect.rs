use trame::term::Terminal;
use unicode_width::UnicodeWidthChar;

use crate::cases::Case;

const ESCAPE: char = '\x1b';

pub fn last_row(input: &[u8], cols: usize) -> String {
    let line = input
        .trim_ascii_end()
        .rsplit(|&byte| byte == b'\n')
        .next()
        .unwrap_or_default();
    let mut chars = std::str::from_utf8(line).expect("cases are UTF-8").chars();
    let (mut row, mut col) = (String::new(), 0);
    while let Some(ch) = chars.next() {
        if ch == ESCAPE {
            chars.find(char::is_ascii_alphabetic);
            continue;
        }
        let width = ch.width().unwrap_or(0);
        if col + width > cols {
            row.clear();
            col = 0;
        }
        row.push(ch);
        col += width;
    }
    row.trim_end().to_owned()
}

pub fn check(terminal: &Terminal, case: &Case) {
    let grid = terminal.screen().grid();
    assert_eq!(grid.history_len(), case.history, "{} history", case.name);
    let row = (0..grid.rows())
        .rev()
        .map(|row| grid.row(row))
        .find(|row| !row.is_blank())
        .expect("every case writes a row");
    assert_eq!(row.text(), case.last_row, "{} last written row", case.name);
    if let Some(probe) = case.probe {
        let cell = row.cells().iter().find(|cell| cell.ch == probe.ch);
        assert_eq!(cell, Some(&probe), "{} cell {:?}", case.name, probe.ch);
    }
}
