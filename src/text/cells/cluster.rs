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
        let span = self.cells.metrics.width * f32::from(key.columns);
        let mut pen = 0.0;
        let mut placed = Vec::new();
        for glyph in shaped {
            if glyph.id != 0
                && let Some(mut found) =
                    self.raster_cell_glyph(placement, glyph.id, pen + glyph.x, glyph.y)
            {
                if !placed.is_empty() {
                    keep_inside(&mut found, span);
                }
                placed.push(found);
            }
            pen += glyph.advance;
        }
        placed
    }
}

fn keep_inside(mark: &mut CellGlyph, span: f32) {
    let width = mark.region.width as f32;
    let center = mark.x + width / 2.0;
    if center <= 0.0 || center >= span {
        mark.x = ((span - width) / 2.0).round();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{AtlasKind, AtlasRegion};

    fn mark_at(x: f32) -> CellGlyph {
        let region = AtlasRegion {
            kind: AtlasKind::Mask,
            x: 0,
            y: 0,
            width: 4,
            height: 3,
        };
        CellGlyph { region, x, y: 0.0 }
    }

    #[test]
    fn a_mark_left_in_the_next_cell_comes_back_over_its_base() {
        let mut mark = mark_at(9.0);
        keep_inside(&mut mark, 10.0);
        assert_eq!(mark.x, 3.0);
    }

    #[test]
    fn a_mark_placed_by_the_font_stays_where_it_is() {
        let mut mark = mark_at(2.0);
        keep_inside(&mut mark, 10.0);
        assert_eq!(mark.x, 2.0);
    }
}
