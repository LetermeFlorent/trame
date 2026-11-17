use std::collections::HashSet;

use super::ImageStore;
use crate::term::grid::Grid;

impl ImageStore {
    pub fn referenced(&self, grids: [&Grid; 2]) -> HashSet<u32> {
        grids
            .into_iter()
            .flat_map(Grid::anchors)
            .map(|anchor| anchor.image)
            .chain(self.virtuals.keys().copied())
            .collect()
    }

    pub fn collect(&mut self, candidates: impl IntoIterator<Item = u32>, grids: [&Grid; 2]) {
        let referenced = self.referenced(grids);
        for id in candidates {
            if !referenced.contains(&id) {
                self.remove(id);
            }
        }
    }

    pub fn evict_unreferenced(&mut self, limit: usize, grids: [&Grid; 2]) {
        if self.bytes() <= limit {
            return;
        }
        let referenced = self.referenced(grids);
        self.evict_until(limit, &|id| referenced.contains(&id));
    }
}
