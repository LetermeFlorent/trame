use std::time::{Duration, Instant};

use trame::term::SelectionKind;

use super::{Cell, Selecting};

const MULTI_CLICK: Duration = Duration::from_millis(500);
pub(super) const CLICK_KINDS: [SelectionKind; 3] = [
    SelectionKind::Simple,
    SelectionKind::Word,
    SelectionKind::Line,
];

impl Selecting {
    pub(super) fn count_click(&mut self, cell: Cell) -> usize {
        let now = Instant::now();
        let count = match self.clicks {
            Some((at, last, count)) if last == cell && now - at < MULTI_CLICK => {
                count % CLICK_KINDS.len() + 1
            }
            _ => 1,
        };
        self.clicks = Some((now, cell, count));
        count
    }
}
