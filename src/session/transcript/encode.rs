use super::style::Style;
use crate::term::grid::Row;
use crate::term::{Cell, Flags};

pub const MAX_TRANSCRIPT_BYTES: usize = 4 << 20;

const LINE_END: &[u8] = b"\r\n";
const RESET: &[u8] = b"\x1b[0m";

pub(super) fn encode(rows: &[Row], cols: usize, budget: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut starts = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        starts.push(out.len());
        encode_row(row, cols, index + 1 == rows.len(), &mut out);
    }
    keep_tail(out, &starts, budget)
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
        for ch in std::iter::once(printable(cell.ch)).chain(cell.marks.chars()) {
            out.extend_from_slice(ch.encode_utf8(&mut buffer).as_bytes());
        }
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
