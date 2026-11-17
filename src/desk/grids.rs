use std::collections::HashMap;

use crate::session::SessionId;

pub const FALLBACK_GRID: (usize, usize) = (24, 80);

#[derive(Clone, Debug, Default)]
pub struct Grids {
    sizes: HashMap<SessionId, (usize, usize)>,
    shown: Vec<SessionId>,
    cell_pixels: (u16, u16),
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

    pub fn set_cell_pixels(&mut self, width: u16, height: u16) {
        self.cell_pixels = (width, height);
    }

    pub fn cell_pixels(&self) -> (u16, u16) {
        self.cell_pixels
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
