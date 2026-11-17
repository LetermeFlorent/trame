use super::diacritics::{PLACEHOLDER, diacritic_index};
use crate::term::cell::{Cell, Color, Marks};
use crate::term::grid::Row;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Raw {
    pub id_low: u32,
    pub row: Option<u16>,
    pub col: Option<u16>,
    pub high: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Run {
    pub id: u32,
    pub img_row: u16,
    pub img_col: u16,
    pub start: usize,
    pub len: usize,
}

#[derive(Clone, Copy)]
struct Resolved {
    fg: Color,
    id_low: u32,
    row: u16,
    col: u16,
    high: u16,
}

pub(crate) fn placeholder_cell(cell: &Cell, marks: Marks) -> Option<Raw> {
    if cell.ch != PLACEHOLDER {
        return None;
    }
    let id_low = match cell.fg {
        Color::Indexed(n) => u32::from(n),
        Color::Rgb(r, g, b) => u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b),
        Color::Foreground | Color::Background => 0,
    };
    let mut found = marks.chars().map(diacritic_index);
    let mut next = || found.next().flatten();
    Some(Raw {
        id_low,
        row: next(),
        col: next(),
        high: next(),
    })
}

pub(crate) fn resolve_row(row: &Row, cols: usize) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    let mut prev: Option<Resolved> = None;
    for (at, cell) in row.cells().iter().enumerate().take(cols) {
        if cell.ch != PLACEHOLDER {
            prev = None;
            continue;
        }
        let Some(raw) = placeholder_cell(cell, row.marks(at)) else {
            continue;
        };
        let current = resolve(raw, cell.fg, prev.filter(|left| left.fg == cell.fg));
        prev = Some(current);
        if current.id_low != 0 {
            extend(&mut runs, &current, at);
        }
    }
    runs
}

fn resolve(raw: Raw, fg: Color, left: Option<Resolved>) -> Resolved {
    let row = raw.row.or(left.map(|left| left.row)).unwrap_or(0);
    let left = left.filter(|left| left.row == row);
    Resolved {
        fg,
        id_low: raw.id_low,
        row,
        col: raw
            .col
            .or(left.map(|left| left.col.saturating_add(1)))
            .unwrap_or(0),
        high: raw.high.or(left.map(|left| left.high)).unwrap_or(0),
    }
}

fn extend(runs: &mut Vec<Run>, cell: &Resolved, at: usize) {
    let id = u32::from(u8::try_from(cell.high).unwrap_or(0)) << 24 | cell.id_low;
    if let Some(last) = runs.last_mut()
        && last.id == id
        && last.img_row == cell.row
        && last.start + last.len == at
        && usize::from(last.img_col) + last.len == usize::from(cell.col)
    {
        last.len += 1;
        return;
    }
    runs.push(Run {
        id,
        img_row: cell.row,
        img_col: cell.col,
        start: at,
        len: 1,
    });
}
