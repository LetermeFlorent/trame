use std::collections::HashMap;

use crate::session::SessionId;

pub const FALLBACK_GRID: (usize, usize) = (24, 80);

#[derive(Clone, Debug, Default)]
pub struct Grids {
    sizes: HashMap<SessionId, (usize, usize)>,
    shown: Vec<SessionId>,
}

impl Grids {
    pub fn add_pane(
        &mut self,
        tabs: &[SessionId],
        active: Option<SessionId>,
        size: (usize, usize),
    ) {
        for tab in tabs {
            self.sizes.entry(*tab).or_insert(size);
        }
        self.shown.extend(active);
    }

    pub fn size_of(&self, id: SessionId) -> (usize, usize) {
        self.sizes.get(&id).copied().unwrap_or(FALLBACK_GRID)
    }

    pub fn shown(&self) -> &[SessionId] {
        &self.shown
    }

    pub fn sizes(&self) -> impl Iterator<Item = (SessionId, (usize, usize))> + '_ {
        self.sizes.iter().map(|(id, size)| (*id, *size))
    }
}
