mod key;
mod upload;

use std::collections::HashMap;

use swash::scale::image::Image;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::{Angle, Format, Transform, Vector};

use super::MAX_CACHED_GLYPHS;
use super::font::{FontKey, FontLibrary, WEIGHT_AXIS};
use crate::render::{Atlas, AtlasRegion};
pub use key::GlyphKey;
use upload::upload;

const SOURCES: [Source; 3] = [
    Source::ColorOutline(0),
    Source::ColorBitmap(StrikeWith::BestFit),
    Source::Outline,
];
const EMBOLDEN_RATIO: f32 = 0.035;
const MIN_EMBOLDEN: f32 = 0.4;
const SKEW_DEGREES: f32 = 12.0;

#[derive(Clone, Copy, Debug)]
pub struct RasterGlyph {
    pub region: AtlasRegion,
    pub left: i32,
    pub top: i32,
}

#[derive(Default)]
pub struct GlyphCache {
    context: ScaleContext,
    glyphs: HashMap<GlyphKey, Option<RasterGlyph>>,
}

impl GlyphCache {
    pub fn get(
        &mut self,
        library: &FontLibrary,
        atlas: &mut Atlas,
        key: GlyphKey,
    ) -> Option<RasterGlyph> {
        if let Some(found) = self.glyphs.get(&key) {
            return *found;
        }
        let raster = match self.rasterize(library, key) {
            Some(image) if image.placement.width > 0 && image.placement.height > 0 => {
                upload(atlas, image)
            }
            _ => None,
        };
        self.remember(atlas, key, raster);
        raster
    }

    fn remember(&mut self, atlas: &mut Atlas, key: GlyphKey, raster: Option<RasterGlyph>) {
        if self.glyphs.len() >= MAX_CACHED_GLYPHS {
            atlas.request_reset();
        }
        self.glyphs.insert(key, raster);
    }

    pub fn clear(&mut self) {
        self.glyphs.clear();
    }

    pub fn ink_width(
        &mut self,
        library: &FontLibrary,
        font: FontKey,
        glyph: u16,
        size: f32,
    ) -> Option<f32> {
        let face = library.face(font.id);
        let mut scaler = self.context.builder(face.font()).size(size).build();
        let outline = scaler
            .scale_color_outline(glyph)
            .or_else(|| scaler.scale_outline(glyph))?;
        Some(outline.bounds().width()).filter(|width| *width > 0.0)
    }

    fn rasterize(&mut self, library: &FontLibrary, key: GlyphKey) -> Option<Image> {
        let face = library.face(key.font.id);
        let mut builder = self
            .context
            .builder(face.font())
            .size(key.size())
            .hint(key.hinted);
        if let Some(weight) = key.font.weight {
            builder = builder.variations([(WEIGHT_AXIS, f32::from(weight))]);
        }
        let mut scaler = builder.build();
        let mut render = Render::new(&SOURCES);
        render
            .format(Format::Alpha)
            .offset(Vector::new(key.offset(), 0.0));
        if key.font.embolden {
            render.embolden((key.size() * EMBOLDEN_RATIO).max(MIN_EMBOLDEN));
        }
        if key.font.skew {
            let skew = Transform::skew(Angle::from_degrees(SKEW_DEGREES), Angle::ZERO);
            render.transform(Some(skew));
        }
        render.render(&mut scaler, key.glyph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_cache_resets_the_atlas_with_it() {
        let font = FontKey {
            id: crate::text::font::FontId::first(),
            weight: None,
            embolden: false,
            skew: false,
        };
        let mut cache = GlyphCache::default();
        let mut atlas = Atlas::new();
        atlas.finish_frame();
        for glyph in 0..=MAX_CACHED_GLYPHS as u16 {
            assert!(!atlas.finish_frame());
            cache.remember(&mut atlas, GlyphKey::hinted(font, glyph, 14.0), None);
        }
        assert!(atlas.finish_frame());
    }
}
