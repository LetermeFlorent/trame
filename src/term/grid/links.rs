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

    pub(crate) fn remap_links(&mut self, map: &[u16]) {
        for row in &mut self.lines {
            row.remap_links(map);
        }
    }
}
