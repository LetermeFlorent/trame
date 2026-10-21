use super::Grid;
use super::selection::{GridPoint, Selection, SelectionKind, Span};
use crate::term::cell::Flags;

impl Grid {
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    pub fn is_selected(&self, visible_row: usize, col: usize) -> bool {
        let line = self.first_visible_line() + visible_row as u64;
        self.selection
            .as_ref()
            .is_some_and(|selection| selection.contains(line, col))
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn start_selection(&mut self, kind: SelectionKind, at: GridPoint) {
        let at = self.clamp_point(at);
        let anchor = self.span_at(kind, at);
        self.selection = Some(Selection::new(kind, anchor));
    }

    pub fn extend_selection(&mut self, to: GridPoint) {
        let to = self.clamp_point(to);
        let Some(kind) = self.selection.as_ref().map(Selection::kind) else {
            return;
        };
        let head = self.span_at(kind, to);
        if let Some(selection) = self.selection.as_mut() {
            selection.extend(head);
        }
    }

    pub(super) fn drop_selection_touching(&mut self, first: u64, last: u64) {
        if self
            .selection
            .as_ref()
            .is_some_and(|selection| selection.touches(first, last))
        {
            self.selection = None;
        }
    }

    fn span_at(&self, kind: SelectionKind, at: GridPoint) -> Span {
        match kind {
            SelectionKind::Simple => Span::single(at),
            SelectionKind::Word => self.word_span(at),
            SelectionKind::Line => self.line_span(at),
        }
    }

    fn clamp_point(&self, at: GridPoint) -> GridPoint {
        let last_line = self.end_line() - 1;
        let line = at.line.clamp(self.first_line(), last_line);
        let mut col = at.col.min(self.cols - 1);
        let on_spacer = self
            .line(line)
            .is_some_and(|row| row.cell(col).flags.contains(Flags::WIDE_SPACER));
        if on_spacer && col > 0 {
            col -= 1;
        }
        GridPoint::new(line, col)
    }
}
