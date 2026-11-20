use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ops::Range;
use std::path::{Path, PathBuf};

use super::{LinkSpan, view_links_in};
use crate::session::{Session, SessionId};
use crate::term::Terminal;

#[derive(Clone, PartialEq, Eq)]
struct Key {
    generation: u64,
    first_line: u64,
    size: (usize, usize),
    rows: (isize, isize),
    cwd: Option<PathBuf>,
}

impl Key {
    fn new(terminal: &Terminal, rows: &Range<isize>, cwd: Option<&Path>) -> Self {
        let grid = terminal.screen().grid();
        Self {
            generation: terminal.generation(),
            first_line: grid.first_visible_line(),
            size: (grid.rows(), grid.cols()),
            rows: (rows.start, rows.end),
            cwd: cwd.map(Path::to_path_buf),
        }
    }

    fn matches(&self, terminal: &Terminal, rows: &Range<isize>, cwd: Option<&Path>) -> bool {
        let grid = terminal.screen().grid();
        self.generation == terminal.generation()
            && self.first_line == grid.first_visible_line()
            && self.size == (grid.rows(), grid.cols())
            && self.rows == (rows.start, rows.end)
            && self.cwd.as_deref() == cwd
    }
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
    pub fn spans(
        &mut self,
        id: SessionId,
        terminal: &Terminal,
        rows: Range<isize>,
        cwd: Option<&Path>,
    ) -> &[LinkSpan] {
        match self.views.entry(id) {
            Entry::Occupied(mut entry) => {
                let view = entry.get_mut();
                if !view.key.matches(terminal, &rows, cwd) {
                    view.key = Key::new(terminal, &rows, cwd);
                    view.spans = view_links_in(terminal.screen(), rows, cwd);
                }
                &entry.into_mut().spans
            }
            Entry::Vacant(entry) => {
                &entry
                    .insert(View {
                        key: Key::new(terminal, &rows, cwd),
                        spans: view_links_in(terminal.screen(), rows, cwd),
                    })
                    .spans
            }
        }
    }

    pub fn keep(&mut self, sessions: &HashMap<SessionId, Session>) {
        self.views.retain(|id, _| sessions.contains_key(id));
    }
}
