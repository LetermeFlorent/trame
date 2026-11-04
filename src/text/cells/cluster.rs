use std::iter;

use swash::text::Codepoint;

use super::{CellGlyph, CellText};
use crate::term::Marks;
use crate::text::insert_bounded;
use crate::text::layout::{Shaping, script_of};
use crate::text::system::TextSystem;

impl TextSystem {
    pub(super) fn cluster_glyphs(&mut self, text: CellText) -> &[CellGlyph] {
        let key = CellText {
            columns: text.columns.max(1),
            ..text
        };
        if !self.cells.clusters.contains_key(&key) {
            let glyphs = self.place_cluster(key);
            insert_bounded(&mut self.cells.clusters, key, glyphs);
        }
        &self.cells.clusters[&key]
    }

    fn place_cluster(&mut self, key: CellText) -> Vec<CellGlyph> {
        let key = composed(key);
        if key.marks.is_empty() {
            return self.cell_glyph(key).into_iter().collect();
        }
        let Some(placement) = self.placement(key) else {
            return Vec::new();
        };
        let cluster: String = iter::once(key.ch).chain(key.marks.chars()).collect();
        let shaped = Shaping {
            context: &mut self.shaper,
            library: &mut self.library,
        }
        .glyphs(placement.font, placement.size, script_of(key.ch), &cluster);
        let mut pen = 0.0;
        let mut placed = Vec::new();
        for glyph in shaped {
            if glyph.id != 0
                && let Some(found) =
                    self.raster_cell_glyph(placement, glyph.id, pen + glyph.x, glyph.y)
            {
                placed.push(found);
            }
            pen += glyph.advance;
        }
        placed
    }
}

fn composed(text: CellText) -> CellText {
    let mut ch = text.ch;
    let mut marks = Marks::default();
    for mark in text.marks.chars() {
        match char::compose(ch, mark).filter(|_| marks.is_empty()) {
            Some(joined) => ch = joined,
            None => marks = marks.with(mark),
        }
    }
    CellText { ch, marks, ..text }
}
