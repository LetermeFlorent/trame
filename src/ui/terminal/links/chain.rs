use std::ops::Range;
use std::path::Path;

use super::paths::find_image_paths;
use super::resolve::resolve_cell_link;
use super::{Found, LinkKind, LinkSpan, find_links};
use crate::term::grid::Grid;
use crate::term::{Cell, Flags, Screen};

const MAX_CHAIN: isize = 64;
const EMPTY_CELL: char = '\0';

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
    last_link: Option<(u16, Option<(LinkKind, String)>)>,
}

pub(super) fn chain_start(grid: &Grid, row: isize) -> isize {
    let mut start = row;
    while row - start < MAX_CHAIN && grid.view_row(start - 1).is_some_and(|line| line.wrapped) {
        start -= 1;
    }
    start
}

pub(super) fn chain_links(
    screen: &Screen,
    start: isize,
    cwd: Option<&Path>,
) -> (isize, Vec<LinkSpan>) {
    chain_links_with(screen, start, cwd, &mut Path::is_file)
}

pub(super) fn chain_links_with(
    screen: &Screen,
    start: isize,
    cwd: Option<&Path>,
    is_file: &mut impl FnMut(&Path) -> bool,
) -> (isize, Vec<LinkSpan>) {
    let grid = screen.grid();
    let cols = grid.cols();
    let mut chain = Chain::default();
    let mut row = start;
    while let Some(line) = grid.view_row(row) {
        let cells = &line.cells()[..cols.min(line.len())];
        chain.add_row(screen, row, cells, is_file);
        if line.wrapped {
            chain.pad(row, cells.len()..cols);
        }
        row += 1;
        if !line.wrapped || row - start >= MAX_CHAIN {
            break;
        }
    }
    let mut claimed = Vec::new();
    for found in find_links(&chain.text) {
        claimed.push((found.start, found.end));
        chain.split(&found, LinkKind::Web);
    }
    for found in find_image_paths(&chain.text, &mut claimed, cwd) {
        chain.split(&found, LinkKind::Image);
    }
    chain.spans.sort_by_key(|s| (s.row, s.start));
    (row, chain.spans)
}

impl Chain {
    fn add_row(
        &mut self,
        screen: &Screen,
        row: isize,
        cells: &[Cell],
        is_file: &mut impl FnMut(&Path) -> bool,
    ) {
        for (col, cell) in cells.iter().enumerate() {
            if cell.link != 0 {
                let id = cell.link;
                if self.last_link.as_ref().is_none_or(|(k, _)| *k != id) {
                    let found = screen
                        .link(id)
                        .and_then(|u| resolve_cell_link(u, &mut *is_file));
                    self.last_link = Some((id, found));
                }
                if let Some((_, Some((kind, target)))) = &self.last_link {
                    mark_span(&mut self.spans, row, col, target, *kind);
                }
            }
            if cell.flags.contains(Flags::WIDE_SPACER) {
                continue;
            }
            let plain = cell.link == 0 && cell.ch != EMPTY_CELL;
            self.text.push(if plain { cell.ch } else { ' ' });
            let width = usize::from(cell.columns());
            self.places.push(Place { row, col, width });
        }
    }

    fn pad(&mut self, row: isize, cols: Range<usize>) {
        for col in cols {
            self.text.push(' ');
            self.places.push(Place { row, col, width: 1 });
        }
    }

    fn split(&mut self, found: &Found, kind: LinkKind) {
        let first = self.spans.len();
        for place in &self.places[found.start..found.end] {
            match self.spans[first..].last_mut() {
                Some(span) if span.row == place.row => span.end = place.col + place.width,
                _ => self.spans.push(LinkSpan {
                    row: place.row,
                    start: place.col,
                    end: place.col + place.width,
                    target: found.target.clone(),
                    kind,
                }),
            }
        }
    }
}

fn mark_span(spans: &mut Vec<LinkSpan>, row: isize, col: usize, target: &str, kind: LinkKind) {
    if let Some(last) = spans.last_mut()
        && last.row == row
        && last.end == col
        && last.kind == kind
        && last.target == target
    {
        last.end = col + 1;
        return;
    }
    spans.push(LinkSpan {
        row,
        start: col,
        end: col + 1,
        target: target.to_owned(),
        kind,
    });
}
