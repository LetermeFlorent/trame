use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use swash::shape::ShapeContext;

use super::cells::CellCache;
use super::font::{self, Family, FontId, FontLibrary};
use super::layout::{FontRole, Shaping, TextLayout, TextStyle};
use super::metrics::CellMetrics;
use super::raster::{GlyphCache, GlyphKey};
use crate::geometry::Point;
use crate::render::{Atlas, Scene};
use crate::theme::Rgba;

type LayoutCache = HashMap<String, Vec<Arc<TextLayout>>>;

#[derive(Debug, Clone, Copy)]
pub struct MissingFont;

impl fmt::Display for MissingFont {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("no usable monospace font was found on this system")
    }
}

impl std::error::Error for MissingFont {}

pub struct TextSystem {
    pub(super) library: FontLibrary,
    pub(super) terminal: Family,
    interface: Family,
    pub(super) atlas: Atlas,
    pub(super) glyphs: GlyphCache,
    pub(super) cells: CellCache,
    shaper: ShapeContext,
    layouts: LayoutCache,
    previous_layouts: LayoutCache,
}

impl TextSystem {
    pub fn new(font_size: f32, line_height: f32) -> Result<Self, MissingFont> {
        let system = font::system_fonts();
        let mut library = FontLibrary::new(&system);
        let mut terminal = library.family(&system.terminal);
        if terminal.is_empty() {
            terminal = library.scan_generic(fontdb::Family::Monospace);
        }
        let primary = terminal.primary(&library).ok_or(MissingFont)?;
        let mut interface = library.family(&system.interface);
        if interface.is_empty() {
            interface = library.scan_generic(fontdb::Family::SansSerif);
        }
        if interface.is_empty() {
            interface = terminal.clone();
        }
        let metrics = CellMetrics::measure(library.face(primary), font_size, line_height);
        Ok(Self {
            library,
            terminal,
            interface,
            atlas: Atlas::new(),
            glyphs: GlyphCache::default(),
            cells: CellCache::new(primary, metrics),
            shaper: ShapeContext::new(),
            layouts: LayoutCache::new(),
            previous_layouts: LayoutCache::new(),
        })
    }

    pub fn atlas_mut(&mut self) -> &mut Atlas {
        &mut self.atlas
    }

    pub fn cell_metrics(&self) -> CellMetrics {
        self.cells.metrics()
    }

    pub fn set_terminal_font(&mut self, font_size: f32, line_height: f32) {
        let primary: FontId = self.cells.primary();
        let metrics = CellMetrics::measure(self.library.face(primary), font_size, line_height);
        self.cells.set_metrics(metrics);
    }

    pub fn layout(&mut self, text: &str, style: TextStyle) -> Arc<TextLayout> {
        if let Some(found) = find_layout(&self.layouts, text, style) {
            return found;
        }
        let layout = find_layout(&self.previous_layouts, text, style).unwrap_or_else(|| {
            let family = match style.role {
                FontRole::Terminal => &self.terminal,
                FontRole::Interface => &self.interface,
            };
            let shaping = Shaping {
                context: &mut self.shaper,
                library: &mut self.library,
            };
            Arc::new(shaping.shape(family, text, style))
        });
        self.layouts
            .entry(text.to_owned())
            .or_default()
            .push(layout.clone());
        layout
    }

    pub fn draw_layout(&mut self, scene: &mut Scene, layout: &TextLayout, pen: Point, color: Rgba) {
        for glyph in &layout.glyphs {
            let position = pen.x + glyph.x;
            let key = GlyphKey::subpixel(glyph.font, glyph.glyph, layout.style.size, position);
            let Some(raster) = self.glyphs.get(&self.library, &mut self.atlas, key) else {
                continue;
            };
            let left = position.floor() + raster.left as f32;
            let top = (pen.y - glyph.y).round() - raster.top as f32;
            scene.glyph(raster.region, left, top, color);
        }
    }

    pub fn end_frame(&mut self) -> bool {
        self.previous_layouts = std::mem::take(&mut self.layouts);
        if !self.atlas.is_full() {
            return false;
        }
        self.atlas.clear();
        self.glyphs.clear();
        self.cells.clear();
        true
    }
}

fn find_layout(cache: &LayoutCache, text: &str, style: TextStyle) -> Option<Arc<TextLayout>> {
    cache
        .get(text)?
        .iter()
        .find(|layout| layout.style == style)
        .cloned()
}
