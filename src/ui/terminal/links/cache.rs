use std::collections::HashMap;
use std::ops::Range;

use super::{LinkSpan, view_links};
use crate::session::{Session, SessionId};
use crate::term::Terminal;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    generation: u64,
    first_line: u64,
    size: (usize, usize),
    rows: (isize, isize),
}

struct View {
    key: Key,
    spans: Vec<LinkSpan>,
}

#[derive(Default)]
pub struct LinkCache {
    views: HashMap<SessionId, View>,
}

impl LinkCache {
    pub fn spans(&mut self, id: SessionId, terminal: &Terminal, rows: Range<isize>) -> &[LinkSpan] {
        let grid = terminal.screen().grid();
        let key = Key {
            generation: terminal.generation(),
            first_line: grid.first_visible_line(),
            size: (grid.rows(), grid.cols()),
            rows: (rows.start, rows.end),
        };
        let view = self.views.entry(id).or_insert_with(|| View {
            key,
            spans: view_links(terminal.screen(), rows.clone()),
        });
        if view.key != key {
            view.key = key;
            view.spans = view_links(terminal.screen(), rows);
        }
        &view.spans
    }

    pub fn keep(&mut self, sessions: &HashMap<SessionId, Session>) {
        self.views.retain(|id, _| sessions.contains_key(id));
    }
}
