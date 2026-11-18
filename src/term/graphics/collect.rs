use std::collections::HashSet;

use super::ImageStore;
use super::store::cost;
use crate::term::grid::Grid;

impl ImageStore {
    pub fn referenced(&self, grids: [&Grid; 2]) -> HashSet<u32> {
        grids
            .into_iter()
            .flat_map(Grid::anchors)
            .map(|anchor| anchor.image)
            .chain(self.virtuals.keys().map(|&(image, _)| image))
            .collect()
    }

    pub fn collect(&mut self, candidates: impl IntoIterator<Item = u32>, grids: [&Grid; 2]) {
        let referenced = self.referenced(grids);
        let doomed: Vec<u32> = candidates
            .into_iter()
            .filter(|id| !referenced.contains(id))
            .collect();
        self.remove_all(&doomed);
    }

    pub fn evict_unreferenced(&mut self, limit: usize, grids: [&Grid; 2]) {
        if self.bytes() <= limit {
            return;
        }
        let referenced = self.referenced(grids);
        self.evict_until(limit, &|id| referenced.contains(&id));
    }

    pub fn evict_until(&mut self, limit: usize, keep: &dyn Fn(u32) -> bool) {
        let mut queue: Vec<(bool, u64, u32, usize)> = self
            .images
            .iter()
            .map(|(&id, image)| (keep(id), image.serial, id, cost(image)))
            .collect();
        queue.sort_unstable();
        let mut left = self.bytes;
        let doomed: Vec<u32> = queue
            .into_iter()
            .take_while(|&(.., size)| {
                let over = left > limit;
                left = left.saturating_sub(size);
                over
            })
            .map(|(_, _, id, _)| id)
            .collect();
        self.remove_all(&doomed);
    }

    pub fn remove_all(&mut self, ids: &[u32]) {
        for id in ids {
            if let Some(old) = self.images.remove(id) {
                self.bytes -= cost(&old);
            }
        }
        let gone = |id: &u32| !self.images.contains_key(id);
        self.numbers.retain(|_, image| !gone(image));
        self.virtuals.retain(|(image, _), _| !gone(image));
    }
}
