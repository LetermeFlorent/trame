use std::collections::HashMap;

use super::font::{BOLD, FontId, REGULAR};
use super::metrics::CellMetrics;
use super::raster::GlyphKey;
use super::system::TextSystem;
use crate::geometry::Point;
use crate::render::{AtlasRegion, Scene};
use crate::theme::Rgba;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellText {
    pub ch: char,
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

pub struct CellCache {
    primary: FontId,
    metrics: CellMetrics,
    glyphs: HashMap<CellText, Option<CellGlyph>>,
}

impl CellCache {
    pub fn new(primary: FontId, metrics: CellMetrics) -> Self {
        Self {
            primary,
            metrics,
            glyphs: HashMap::new(),
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
        if found.is_some() || !self.atlas.is_full() {
            self.cells.glyphs.insert(key, found);
        }
        found
    }

    pub fn draw_cell(&mut self, scene: &mut Scene, text: CellText, origin: Point, color: Rgba) {
        if let Some(glyph) = self.cell_glyph(text) {
            scene.glyph(glyph.region, origin.x + glyph.x, origin.y + glyph.y, color);
        }
    }

    fn place_cell_glyph(&mut self, key: CellText) -> Option<CellGlyph> {
        let weight = if key.bold { BOLD } else { REGULAR };
        let primary = self.terminal.resolve(&self.library, weight, key.italic)?;
        let font = self
            .library
            .covering_font(primary, key.ch, weight, key.italic)?;
        let glyph = self.library.face(font.id).glyph(key.ch);
        let metrics = self.cells.metrics;
        let span = metrics.width * f32::from(key.columns);
        let mut size = metrics.font_size;
        let mut offset = 0.0;
        if font.id != primary.id {
            let advance = self.library.face(font.id).advance(glyph, size);
            let ink = self
                .glyphs
                .ink_width(&self.library, font, glyph, size)
                .unwrap_or(advance);
            let scale = if ink > span { span / ink } else { 1.0 };
            size *= scale;
            offset = ((span - advance * scale) / 2.0).round();
        }
        let glyph_key = GlyphKey::hinted(font, glyph, size);
        let raster = self.glyphs.get(&self.library, &mut self.atlas, glyph_key)?;
        Some(CellGlyph {
            region: raster.region,
            x: offset + raster.left as f32,
            y: metrics.baseline - raster.top as f32,
        })
    }
}
