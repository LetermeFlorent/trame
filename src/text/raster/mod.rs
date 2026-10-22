mod key;

use std::collections::HashMap;

use swash::scale::image::{Content, Image};
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::{Angle, Format, Transform, Vector};

use super::font::{FontKey, FontLibrary, WEIGHT_AXIS};
use crate::render::{Atlas, AtlasKind, AtlasRegion};
pub use key::GlyphKey;

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
                Some(upload(atlas, image)?)
            }
            _ => None,
        };
        self.glyphs.insert(key, raster);
        raster
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

fn upload(atlas: &mut Atlas, image: Image) -> Option<RasterGlyph> {
    let kind = match image.content {
        Content::Color => AtlasKind::Color,
        Content::Mask | Content::SubpixelMask => AtlasKind::Mask,
    };
    let data = match image.source {
        Source::ColorOutline(_) => unpremultiply(image.data),
        _ => image.data,
    };
    let region = atlas.insert(kind, image.placement.width, image.placement.height, data)?;
    Some(RasterGlyph {
        region,
        left: image.placement.left,
        top: image.placement.top,
    })
}

fn unpremultiply(mut data: Vec<u8>) -> Vec<u8> {
    for pixel in data.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha == 0 || alpha == 255 {
            continue;
        }
        for channel in &mut pixel[..3] {
            *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
        }
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premultiplied_colors_are_restored() {
        let restored = unpremultiply(vec![64, 32, 0, 128, 10, 20, 30, 0, 1, 2, 3, 255]);
        assert_eq!(restored, vec![128, 64, 0, 128, 10, 20, 30, 0, 1, 2, 3, 255]);
    }
}
