use trame::session::{transcript_of, transcript_within};
use trame::term::{Cell, Color, Flags, Terminal};

const ROWS: usize = 6;
const COLS: usize = 20;
const HISTORY: usize = 100;

fn terminal_with(bytes: &[u8]) -> Terminal {
    let mut terminal = Terminal::new(ROWS, COLS, HISTORY);
    terminal.feed(bytes);
    terminal
}

fn restored(terminal: &Terminal) -> Terminal {
    terminal_with(&transcript_of(terminal))
}

fn written_lines(terminal: &Terminal) -> Vec<String> {
    let grid = terminal.screen().grid();
    let mut lines: Vec<String> = (grid.first_line()..grid.end_line())
        .filter_map(|line| grid.line(line))
        .map(|row| row.text())
        .collect();
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

fn first_row_cells(terminal: &Terminal, count: usize) -> Vec<Cell> {
    let row = terminal.screen().grid().row(0);
    (0..count).map(|col| row.cell(col)).collect()
}

fn numbered_lines(count: usize) -> Vec<u8> {
    (0..count)
        .flat_map(|index| format!("line {index}\r\n").into_bytes())
        .collect()
}

#[test]
fn plain_text_and_scrollback_come_back() {
    let terminal = terminal_with(&numbered_lines(10));
    let lines = written_lines(&terminal);
    assert_eq!(lines.len(), 10);
    assert_eq!(written_lines(&restored(&terminal)), lines);
}

#[test]
fn colors_and_flags_come_back() {
    let terminal = terminal_with(
        b"\x1b[1;31mred\x1b[0m \x1b[4;48;5;200mmark\x1b[0m \x1b[3;9;38;2;1;2;3mrgb\x1b[0m",
    );
    let width = "red mark rgb".len();
    assert_eq!(
        first_row_cells(&restored(&terminal), width),
        first_row_cells(&terminal, width)
    );
}

#[test]
fn styles_do_not_leak_into_the_next_row() {
    let terminal = terminal_with(b"\x1b[7;32mlit\x1b[0m\r\nplain");
    let copy = restored(&terminal);
    let cell = copy.screen().grid().row(1).cell(0);
    assert_eq!(cell.ch, 'p');
    assert_eq!(cell.fg, Color::Foreground);
    assert_eq!(cell.flags, Flags::empty());
}

#[test]
fn wrapped_rows_stay_one_logical_line() {
    let text = "abcdefghijklmnopqrstuvwxyz0123";
    let terminal = terminal_with(text.as_bytes());
    let transcript = transcript_of(&terminal);
    let line_ends = transcript.windows(2).filter(|pair| pair == b"\r\n").count();
    assert_eq!(line_ends, 1);
    let copy = restored(&terminal);
    assert!(copy.screen().grid().row(0).wrapped);
    assert_eq!(written_lines(&copy), written_lines(&terminal));
}

#[test]
fn wide_characters_keep_their_width() {
    let terminal = terminal_with("\u{6f22}\u{5b57} ok".as_bytes());
    let copy = restored(&terminal);
    assert_eq!(first_row_cells(&copy, 7), first_row_cells(&terminal, 7));
    assert!(
        copy.screen()
            .grid()
            .row(0)
            .cell(0)
            .flags
            .contains(Flags::WIDE)
    );
}

#[test]
fn budget_keeps_the_newest_whole_rows() {
    let terminal = terminal_with(&numbered_lines(10));
    assert_eq!(
        transcript_within(&terminal, 20),
        b"line 8\r\nline 9\r\n".to_vec()
    );
}

#[test]
fn a_row_larger_than_the_budget_gives_nothing() {
    let terminal = terminal_with(b"0123456789");
    assert!(transcript_within(&terminal, 5).is_empty());
}

#[test]
fn an_empty_terminal_gives_no_text() {
    assert!(transcript_of(&Terminal::new(ROWS, COLS, HISTORY)).is_empty());
    assert!(transcript_of(&terminal_with(b"\r\n\r\n   ")).is_empty());
}

#[test]
fn combining_marks_survive_a_restore() {
    let lines = ["e\u{301}x", "\u{1F468}\u{200D}\u{1F469}", " \u{301}"];
    let terminal = terminal_with(lines.join("\r\n").as_bytes());
    assert_eq!(written_lines(&restored(&terminal)), lines);
}

#[test]
fn generation_moves_when_output_arrives() {
    let mut terminal = Terminal::new(ROWS, COLS, HISTORY);
    let before = terminal.generation();
    terminal.feed(b"x");
    assert_ne!(terminal.generation(), before);
}
