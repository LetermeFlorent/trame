use super::style::Style;
use crate::term::grid::{Grid, Row};
use crate::term::{Cell, Flags, Terminal};

pub const MAX_TRANSCRIPT_BYTES: usize = 4 << 20;

const LINE_END: &[u8] = b"\r\n";
const RESET: &[u8] = b"\x1b[0m";

pub fn transcript_of(terminal: &Terminal) -> Vec<u8> {
    transcript_within(terminal, MAX_TRANSCRIPT_BYTES)
}

pub fn transcript_within(terminal: &Terminal, budget: usize) -> Vec<u8> {
    encode(&terminal.screen().primary, budget)
}

fn encode(grid: &Grid, budget: usize) -> Vec<u8> {
    let Some(last) = last_written_line(grid) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut starts = Vec::new();
    for line in grid.first_line()..=last {
        if let Some(row) = grid.line(line) {
            starts.push(out.len());
            encode_row(row, grid.cols(), line == last, &mut out);
        }
    }
    keep_tail(out, &starts, budget)
}

fn last_written_line(grid: &Grid) -> Option<u64> {
    (grid.first_line()..grid.end_line())
        .rev()
        .find(|line| grid.line(*line).is_some_and(|row| !row.is_blank()))
}

fn encode_row(row: &Row, cols: usize, last: bool, out: &mut Vec<u8>) {
    let joined = row.wrapped && !last;
    let cells = row.cells();
    let end = match joined {
        true => cells.len(),
        false => written_width(cells),
    };
    let mut style = Style::PLAIN;
    for cell in cells[..end]
        .iter()
        .filter(|cell| !cell.flags.contains(Flags::WIDE_SPACER))
    {
        let next = Style::of(cell);
        if next != style {
            next.write(out);
            style = next;
        }
        let mut buffer = [0; 4];
        out.extend_from_slice(printable(cell.ch).encode_utf8(&mut buffer).as_bytes());
    }
    if style != Style::PLAIN {
        out.extend_from_slice(RESET);
    }
    match joined {
        true => out.resize(out.len() + cols.saturating_sub(cells.len()), b' '),
        false => out.extend_from_slice(LINE_END),
    }
}

fn written_width(cells: &[Cell]) -> usize {
    cells
        .iter()
        .rposition(|cell| !cell.is_blank())
        .map_or(0, |index| index + 1)
}

fn printable(ch: char) -> char {
    match ch {
        '\0'..=' ' | '\x7f' => ' ',
        _ => ch,
    }
}

fn keep_tail(mut out: Vec<u8>, starts: &[usize], budget: usize) -> Vec<u8> {
    if out.len() <= budget {
        return out;
    }
    let cut = starts
        .iter()
        .copied()
        .find(|start| out.len() - start <= budget)
        .unwrap_or(out.len());
    out.drain(..cut);
    out
}
