use std::collections::HashSet;

use super::Grid;

impl Grid {
    pub(crate) fn mark_links(&self, used: &mut [bool]) {
        let cells = self.lines.iter().flat_map(|row| row.cells());
        for cell in cells.filter(|cell| cell.link != 0) {
            if let Some(slot) = used.get_mut(usize::from(cell.link)) {
                *slot = true;
            }
        }
    }

    pub(crate) fn mark_images(&self, used: &mut HashSet<u32>) {
        let cells = self.lines.iter().flat_map(|row| row.cells());
        for id in cells.filter_map(|cell| cell.image_id()) {
            used.insert(id);
        }
    }

    pub(crate) fn remap_links(&mut self, map: &[u16]) {
        for row in &mut self.lines {
            row.remap_links(map);
        }
    }
}
