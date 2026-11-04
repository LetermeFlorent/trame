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
        let Some(wanted) = self.class_at(at) else {
            return Span::single(at);
        };
        let same = |point: &GridPoint| self.class_at(*point) == Some(wanted);
        let mut start = at;
        while let Some(before) = self.before(start).filter(same) {
            start = before;
        }
        let mut end = at;
        while let Some(after) = self.after(end).filter(same) {
            end = after;
        }
        Span { start, end }
    }

    fn class_at(&self, at: GridPoint) -> Option<Class> {
        let cell = self.line(at.line)?.cell(at.col);
        Some(match cell.ch {
            ' ' | '\0' if !cell.flags.contains(Flags::WIDE_SPACER) => Class::Blank,
            ch if SEPARATORS.contains(ch) => Class::Separator,
            _ => Class::Word,
        })
    }

    fn before(&self, at: GridPoint) -> Option<GridPoint> {
        if at.col > 0 {
            return Some(GridPoint::new(at.line, at.col - 1));
        }
        let line = at.line.checked_sub(1)?;
        self.wraps(line)
            .then(|| GridPoint::new(line, self.cols - 1))
    }

    fn after(&self, at: GridPoint) -> Option<GridPoint> {
        if at.col + 1 < self.cols {
            return Some(GridPoint::new(at.line, at.col + 1));
        }
        let wraps = self.wraps(at.line) && at.line + 1 < self.end_line();
        wraps.then(|| GridPoint::new(at.line + 1, 0))
    }

    fn wraps(&self, line: u64) -> bool {
        line >= self.first_line() && self.line(line).is_some_and(|row| row.wrapped)
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
