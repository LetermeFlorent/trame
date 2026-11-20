mod cache;
mod chain;
mod detect;
mod paths;
mod resolve;
#[cfg(test)]
mod tests;

use std::ops::Range;
use std::path::Path;

use crate::term::Screen;
pub use cache::LinkCache;
use chain::{chain_links, chain_start};
pub use detect::{Found, find_links};
pub use resolve::resolve_image_path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkKind {
    Web,
    Image,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkSpan {
    pub row: isize,
    pub start: usize,
    pub end: usize,
    pub target: String,
    pub kind: LinkKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkHit {
    pub target: String,
    pub kind: LinkKind,
    pub spans: Vec<LinkSpan>,
}

impl LinkSpan {
    pub fn covers(&self, col: usize) -> bool {
        (self.start..self.end).contains(&col)
    }
}

pub fn view_links(screen: &Screen, rows: Range<isize>) -> Vec<LinkSpan> {
    view_links_in(screen, rows, screen.cwd().map(Path::new))
}

pub fn view_links_in(screen: &Screen, rows: Range<isize>, cwd: Option<&Path>) -> Vec<LinkSpan> {
    let cwd = cwd.or_else(|| screen.cwd().map(Path::new));
    let mut spans = Vec::new();
    let mut start = chain_start(screen.grid(), rows.start);
    while start < rows.end {
        let (next, found) = chain_links(screen, start, cwd);
        spans.extend(found.into_iter().filter(|span| rows.contains(&span.row)));
        start = next.max(start + 1);
    }
    spans.sort_by_key(|span| (span.row, span.start));
    spans
}

pub fn link_at(screen: &Screen, row: isize, col: usize) -> Option<String> {
    link_hit(screen, row, col, None).map(|hit| hit.target)
}

pub fn link_hit(screen: &Screen, row: isize, col: usize, cwd: Option<&Path>) -> Option<LinkHit> {
    let cwd = cwd.or_else(|| screen.cwd().map(Path::new));
    let (_, spans) = chain_links(screen, chain_start(screen.grid(), row), cwd);
    let matched = spans
        .iter()
        .find(|span| span.row == row && span.covers(col))?
        .clone();
    let group = spans
        .into_iter()
        .filter(|span| span.kind == matched.kind && span.target == matched.target)
        .collect();
    Some(LinkHit {
        target: matched.target,
        kind: matched.kind,
        spans: group,
    })
}
