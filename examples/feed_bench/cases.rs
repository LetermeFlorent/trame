use std::io::{self, Write};

use trame::term::{Cell, Flags};

use crate::{expect, styled};

pub struct Case {
    pub name: &'static str,
    pub input: Vec<u8>,
    pub history: usize,
    pub last_row: String,
    pub probe: Option<Cell>,
}

pub const LINES: usize = 40_000;
pub const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub const DIGITS: &str = "0123456789";
const WRAPPED_LINES: usize = LINES / 2;
const COLUMN_STEP: usize = 7;
const MARKER: &str = "Final";
const MARKER_MOVES: &str = "fin\x1b[3DF\x1b[2C\x1b[A\x1b[Bal";
const FIXED_LINES: usize = 1;
const WIDE: &str = "日本語の文字 中文字符 한국어 \u{1F600}\u{1F389}";
const COMBINING: &str = "e\u{301}a\u{308}o\u{302}n\u{303}";
const NARROW: &str = "café αβγδ кириллица";
const LENGTH_STEP: usize = 37;

pub fn all(rows: usize, cols: usize, history: usize) -> [Case; 7] {
    let lines = |name, input, history, probe| line_case(name, input, history, cols, probe);
    [
        lines("plain", plain_lines(LINES), history, None),
        lines(
            "sgr",
            styled::sgr_lines(),
            history,
            Some(styled::sgr_probe()),
        ),
        lines(
            "sgr-dense",
            styled::dense_lines(),
            history,
            Some(styled::dense_probe()),
        ),
        Case {
            name: "cursor",
            input: cursor_moves(rows, cols),
            history: 0,
            last_row: MARKER.to_owned(),
            probe: None,
        },
        lines("region", region_scroll(rows), 0, None),
        lines("unicode", unicode_lines(), history, Some(wide_probe())),
        lines("varied", varied_lines(LINES, cols), history, None),
    ]
}

fn line_case(
    name: &'static str,
    input: Vec<u8>,
    history: usize,
    cols: usize,
    probe: Option<Cell>,
) -> Case {
    let last_row = expect::last_row(&input, cols);
    Case {
        name,
        input,
        history,
        last_row,
        probe,
    }
}

pub fn plain_lines(count: usize) -> Vec<u8> {
    generate(count, |out, index| {
        write!(out, "ligne {index} {ALPHABET} {DIGITS} {ALPHABET}\r\n")
    })
}

pub fn varied_lines(count: usize, cols: usize) -> Vec<u8> {
    let span = cols - count.to_string().len() - 1;
    let tail = ALPHABET.repeat(span / ALPHABET.len() + 1);
    let input = generate(count, |out, index| {
        write!(out, "{index} {}\r\n", &tail[..index * LENGTH_STEP % span])
    });
    let longest = longest_line(&input);
    assert!(
        longest < cols,
        "varied line of {longest} bytes wraps at {cols} columns"
    );
    input
}

fn longest_line(input: &[u8]) -> usize {
    input
        .split(|&byte| byte == b'\n')
        .map(|line| line.len().saturating_sub(1))
        .max()
        .unwrap_or(0)
}

fn cursor_moves(rows: usize, cols: usize) -> Vec<u8> {
    let span = cols / 2;
    let mut input = generate(LINES, |out, index| {
        let (row, col) = (index % rows + 1, index * COLUMN_STEP % span + 1);
        write!(out, "\x1b[{row};{col}Hab\x1b[Acd\x1b[2Cef\x1b[Bgh\x1b[3Dij")
    });
    input.extend_from_slice(format!("\x1b[{rows};1H\x1b[2K{MARKER_MOVES}").as_bytes());
    input
}

fn region_scroll(rows: usize) -> Vec<u8> {
    let (top, bottom) = (1 + FIXED_LINES, rows - FIXED_LINES);
    let mut input = format!("\x1b[{top};{bottom}r\x1b[{bottom};1H").into_bytes();
    input.append(&mut plain_lines(LINES));
    input
}

fn unicode_lines() -> Vec<u8> {
    generate(WRAPPED_LINES, |out, index| {
        write!(
            out,
            "ligne {index} {NARROW}{COMBINING} {WIDE}{WIDE}{WIDE}\r\n"
        )
    })
}

fn wide_probe() -> Cell {
    Cell {
        ch: WIDE.chars().next_back().expect("WIDE is not empty"),
        flags: Flags::WIDE,
        ..Cell::default()
    }
}

pub fn generate(
    count: usize,
    write_line: impl Fn(&mut Vec<u8>, usize) -> io::Result<()>,
) -> Vec<u8> {
    let mut out = Vec::new();
    for index in 0..count {
        write_line(&mut out, index).expect("a Vec grows without failing");
    }
    out
}
