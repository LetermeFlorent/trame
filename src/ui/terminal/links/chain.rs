use std::ops::Range;

use super::{Found, LinkSpan, find_links};
use crate::files::LinkTarget;
use crate::term::grid::Grid;
use crate::term::{Cell, Flags, Screen};

const MAX_CHAIN: isize = 64;

#[derive(Clone, Copy)]
struct Place {
    row: isize,
    col: usize,
    width: usize,
}

#[derive(Default)]
struct Chain {
    text: Vec<char>,
    places: Vec<Place>,
    spans: Vec<LinkSpan>,
}

pub(super) fn chain_start(grid: &Grid, row: isize) -> isize {
    let mut start = row;
    while row - start < MAX_CHAIN && grid.view_row(start - 1).is_some_and(|line| line.wrapped) {
        start -= 1;
    }
    start
}

pub(super) fn chain_links(screen: &Screen, start: isize) -> (isize, Vec<LinkSpan>) {
    let grid = screen.grid();
    let cols = grid.cols();
    let mut chain = Chain::default();
    let mut row = start;
    while let Some(line) = grid.view_row(row) {
        let cells = &line.cells()[..cols.min(line.len())];
        chain.add_row(screen, row, cells);
        if line.wrapped {
            chain.pad(row, cells.len()..cols);
        }
        row += 1;
        if !line.wrapped || row - start >= MAX_CHAIN {
            break;
        }
    }
    for found in find_links(&chain.text) {
        chain.split(&found);
    }
    (row, chain.spans)
}

impl Chain {
    fn add_row(&mut self, screen: &Screen, row: isize, cells: &[Cell]) {
        for (col, cell) in cells.iter().enumerate() {
            if let Some(target) = screen.link(cell.link) {
                self.mark(row, col, target);
            }
            if cell.flags.contains(Flags::WIDE_SPACER) {
                continue;
            }
            let plain = cell.link == 0 && cell.ch != '\0';
            self.text.push(if plain { cell.ch } else { ' ' });
            let width = if cell.flags.contains(Flags::WIDE) {
                2
            } else {
                1
            };
            self.places.push(Place { row, col, width });
        }
    }

    fn mark(&mut self, row: isize, col: usize, uri: &str) {
        if let Some(LinkSpan {
            row: last_row,
            end,
            target: LinkTarget::Uri(known),
            ..
        }) = self.spans.last_mut()
            && *last_row == row
            && *end == col
            && known == uri
        {
            *end = col + 1;
            return;
        }
        let target = LinkTarget::Uri(uri.to_owned());
        let (start, end) = (col, col + 1);
        self.spans.push(LinkSpan {
            row,
            start,
            end,
            target,
        });
    }

    fn pad(&mut self, row: isize, cols: Range<usize>) {
        for col in cols {
            self.text.push(' ');
            self.places.push(Place { row, col, width: 1 });
        }
    }

    fn split(&mut self, found: &Found) {
        let first = self.spans.len();
        for place in &self.places[found.start..found.end] {
            match self.spans[first..].last_mut() {
                Some(span) if span.row == place.row => span.end = place.col + place.width,
                _ => self.spans.push(LinkSpan {
                    row: place.row,
                    start: place.col,
                    end: place.col + place.width,
                    target: found.target.clone(),
                }),
            }
        }
    }
}
