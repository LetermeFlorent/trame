use super::Grid;
use crate::term::graphics::Anchor;

impl Grid {
    pub fn anchors(&self) -> impl Iterator<Item = &Anchor> {
        self.lines.iter().flat_map(|row| row.anchors())
    }

    pub fn history_anchors(&self) -> impl Iterator<Item = &Anchor> {
        self.lines
            .range(..self.screen_start())
            .flat_map(|row| row.anchors())
    }

    pub fn remove_anchors(
        &mut self,
        history: bool,
        doomed: impl Fn(isize, &Anchor) -> bool,
    ) -> Vec<u32> {
        let start = self.screen_start();
        let first = if history { 0 } else { start };
        let mut removed = Vec::new();
        for (index, row) in self.lines.iter_mut().enumerate().skip(first) {
            let at = index as isize - start as isize;
            if !row.anchors().iter().any(|anchor| doomed(at, anchor)) {
                continue;
            }
            removed.extend(
                row.anchors()
                    .iter()
                    .filter(|anchor| doomed(at, anchor))
                    .map(|anchor| anchor.image),
            );
            row.retain_anchors(|anchor| !doomed(at, anchor));
        }
        removed
    }
}
