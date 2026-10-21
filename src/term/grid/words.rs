use super::Grid;
use super::selection::{GridPoint, Span};
use crate::term::cell::Flags;

const SEPARATORS: &str = ",\u{2502}`|:\"'()[]{}<>";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Blank,
    Separator,
    Word,
}

impl Grid {
    pub(super) fn word_span(&self, at: GridPoint) -> Span {
        let Some(row) = self.line(at.line) else {
            return Span::single(at);
        };
        let class = |col: usize| {
            let cell = row.cell(col);
            match cell.ch {
                ' ' | '\0' if !cell.flags.contains(Flags::WIDE_SPACER) => Class::Blank,
                ch if SEPARATORS.contains(ch) => Class::Separator,
                _ => Class::Word,
            }
        };
        let wanted = class(at.col);
        let mut start = at.col;
        while start > 0 && class(start - 1) == wanted {
            start -= 1;
        }
        let mut end = at.col;
        while end + 1 < self.cols && class(end + 1) == wanted {
            end += 1;
        }
        Span {
            start: GridPoint::new(at.line, start),
            end: GridPoint::new(at.line, end),
        }
    }

    pub(super) fn line_span(&self, at: GridPoint) -> Span {
        let mut first = at.line;
        while first > self.first_line() && self.line(first - 1).is_some_and(|row| row.wrapped) {
            first -= 1;
        }
        let mut last = at.line;
        while last + 1 < self.end_line() && self.line(last).is_some_and(|row| row.wrapped) {
            last += 1;
        }
        Span {
            start: GridPoint::new(first, 0),
            end: GridPoint::new(last, self.cols - 1),
        }
    }
}
