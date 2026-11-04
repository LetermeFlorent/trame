mod cluster;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use super::font::{BOLD, FontId, FontKey, REGULAR};
use super::insert_bounded;
use super::metrics::CellMetrics;
use super::raster::GlyphKey;
use super::system::TextSystem;
use crate::geometry::Point;
use crate::render::{AtlasRegion, Scene};
use crate::term::Marks;
use crate::theme::Rgba;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellText {
    pub ch: char,
    pub marks: Marks,
    pub bold: bool,
    pub italic: bool,
    pub columns: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct CellGlyph {
    pub region: AtlasRegion,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
struct Placement {
    font: FontKey,
    size: f32,
    offset: f32,
}

pub struct CellCache {
    primary: FontId,
    metrics: CellMetrics,
    glyphs: HashMap<CellText, Option<CellGlyph>>,
    clusters: HashMap<CellText, Vec<CellGlyph>>,
}

impl CellCache {
    pub fn new(primary: FontId, metrics: CellMetrics) -> Self {
        Self {
            primary,
            metrics,
            glyphs: HashMap::new(),
            clusters: HashMap::new(),
        }
    }

    pub fn primary(&self) -> FontId {
        self.primary
    }

    pub fn metrics(&self) -> CellMetrics {
        self.metrics
    }

    pub fn set_metrics(&mut self, metrics: CellMetrics) {
        self.metrics = metrics;
    }

    pub fn clear(&mut self) {
        self.glyphs.clear();
        self.clusters.clear();
    }
}

impl TextSystem {
    pub fn cell_glyph(&mut self, text: CellText) -> Option<CellGlyph> {
        let key = CellText {
            columns: text.columns.max(1),
            ..text
        };
        if let Some(found) = self.cells.glyphs.get(&key) {
            return *found;
        }
        let found = self.place_cell_glyph(key);
        insert_bounded(&mut self.cells.glyphs, key, found);
        found
    }

    pub fn draw_cell(&mut self, scene: &mut Scene, text: CellText, origin: Point, color: Rgba) {
        let single;
        let glyphs = if text.marks.is_empty() {
            single = self.cell_glyph(text);
            single.as_slice()
        } else {
            self.cluster_glyphs(text)
        };
        for glyph in glyphs {
            scene.glyph(glyph.region, origin.x + glyph.x, origin.y + glyph.y, color);
        }
    }

    fn place_cell_glyph(&mut self, key: CellText) -> Option<CellGlyph> {
        let placement = self.placement(key)?;
        let glyph = self.library.face(placement.font.id).glyph(key.ch);
        self.raster_cell_glyph(placement, glyph, 0.0, 0.0)
    }

    fn placement(&mut self, key: CellText) -> Option<Placement> {
        let weight = if key.bold { BOLD } else { REGULAR };
        let primary = self.terminal.resolve(&self.library, weight, key.italic)?;
        let font = self
            .library
            .covering_font(primary, key.ch, weight, key.italic)?;
        let metrics = self.cells.metrics;
        let mut placement = Placement {
            font,
            size: metrics.font_size,
            offset: 0.0,
        };
        if font.id != primary.id {
            let glyph = self.library.face(font.id).glyph(key.ch);
            let span = metrics.width * f32::from(key.columns);
            let advance = self.library.face(font.id).advance(glyph, placement.size);
            let ink = self
                .glyphs
                .ink_width(&self.library, font, glyph, placement.size)
                .unwrap_or(advance);
            let scale = if ink > span { span / ink } else { 1.0 };
            placement.size *= scale;
            placement.offset = ((span - advance * scale) / 2.0).round();
        }
        Some(placement)
    }

    fn raster_cell_glyph(
        &mut self,
        placement: Placement,
        glyph: u16,
        x: f32,
        y: f32,
    ) -> Option<CellGlyph> {
        let key = GlyphKey::hinted(placement.font, glyph, placement.size);
        let raster = self.glyphs.get(&self.library, &mut self.atlas, key)?;
        Some(CellGlyph {
            region: raster.region,
            x: placement.offset + x.round() + raster.left as f32,
            y: self.cells.metrics.baseline - y.round() - raster.top as f32,
        })
    }
}
