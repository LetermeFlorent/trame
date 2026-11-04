mod cache;
mod chain;
mod detect;
mod paths;

use std::ops::Range;

use crate::files::LinkTarget;
use crate::term::Screen;
pub use cache::LinkCache;
use chain::{chain_links, chain_start};
pub use detect::{Found, find_links};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkSpan {
    pub row: isize,
    pub start: usize,
    pub end: usize,
    pub target: LinkTarget,
}

impl LinkSpan {
    pub fn covers(&self, col: usize) -> bool {
        (self.start..self.end).contains(&col)
    }
}

pub fn view_links(screen: &Screen, rows: Range<isize>) -> Vec<LinkSpan> {
    let mut spans = Vec::new();
    let mut start = chain_start(screen.grid(), rows.start);
    while start < rows.end {
        let (next, found) = chain_links(screen, start);
        spans.extend(found.into_iter().filter(|span| rows.contains(&span.row)));
        start = next.max(start + 1);
    }
    spans.sort_by_key(|span| (span.row, span.start));
    spans
}

pub fn link_at(screen: &Screen, row: isize, col: usize) -> Option<LinkTarget> {
    let (_, spans) = chain_links(screen, chain_start(screen.grid(), row));
    spans
        .into_iter()
        .find(|span| span.row == row && span.covers(col))
        .map(|span| span.target)
}
