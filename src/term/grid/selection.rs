#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridPoint {
    pub line: u64,
    pub col: usize,
}

impl GridPoint {
    pub const fn new(line: u64, col: usize) -> Self {
        Self { line, col }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionKind {
    Simple,
    Word,
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: GridPoint,
    pub end: GridPoint,
}

impl Span {
    pub const fn single(point: GridPoint) -> Self {
        Self {
            start: point,
            end: point,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    kind: SelectionKind,
    anchor: Span,
    span: Span,
}

impl Selection {
    pub fn new(kind: SelectionKind, anchor: Span) -> Self {
        Self {
            kind,
            anchor,
            span: anchor,
        }
    }

    pub fn kind(&self) -> SelectionKind {
        self.kind
    }

    pub fn start(&self) -> GridPoint {
        self.span.start
    }

    pub fn end(&self) -> GridPoint {
        self.span.end
    }

    pub fn is_empty(&self) -> bool {
        self.kind == SelectionKind::Simple && self.span.start == self.span.end
    }

    pub fn extend(&mut self, head: Span) {
        self.span = Span {
            start: self.anchor.start.min(head.start),
            end: self.anchor.end.max(head.end),
        };
    }

    pub fn contains(&self, line: u64, col: usize) -> bool {
        if self.is_empty() {
            return false;
        }
        let point = GridPoint::new(line, col);
        self.span.start <= point && point <= self.span.end
    }

    pub fn touches(&self, first: u64, last: u64) -> bool {
        self.span.start.line <= last && self.span.end.line >= first
    }
}
