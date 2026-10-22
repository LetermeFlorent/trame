use crate::text::font::FontKey;

const SUBPIXEL_BINS: u8 = 3;
const SIZE_UNITS: f32 = 64.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub(super) font: FontKey,
    pub(super) glyph: u16,
    size: u32,
    subpixel: u8,
    pub(super) hinted: bool,
}

impl GlyphKey {
    pub fn hinted(font: FontKey, glyph: u16, size: f32) -> Self {
        Self {
            font,
            glyph,
            size: (size * SIZE_UNITS).round() as u32,
            subpixel: 0,
            hinted: true,
        }
    }

    pub fn subpixel(font: FontKey, glyph: u16, size: f32, x: f32) -> Self {
        let bin = (((x - x.floor()) * f32::from(SUBPIXEL_BINS)) as u8).min(SUBPIXEL_BINS - 1);
        Self {
            subpixel: bin,
            hinted: false,
            ..Self::hinted(font, glyph, size)
        }
    }

    pub(super) fn size(&self) -> f32 {
        self.size as f32 / SIZE_UNITS
    }

    pub(super) fn offset(&self) -> f32 {
        f32::from(self.subpixel) / f32::from(SUBPIXEL_BINS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> FontKey {
        FontKey {
            id: crate::text::font::FontId::first(),
            weight: None,
            embolden: false,
            skew: false,
        }
    }

    fn key(x: f32) -> GlyphKey {
        GlyphKey::subpixel(font(), 1, 14.0, x)
    }

    #[test]
    fn subpixel_bins_follow_the_fraction() {
        assert_eq!(key(10.0).subpixel, 0);
        assert_eq!(key(10.4).subpixel, 1);
        assert_eq!(key(10.99).subpixel, 2);
        assert_eq!(key(-0.5).subpixel, 1);
    }

    #[test]
    fn hinted_glyphs_sit_on_whole_pixels() {
        let key = GlyphKey::hinted(font(), 1, 14.0);
        assert_eq!(key.subpixel, 0);
        assert_eq!(key.offset(), 0.0);
        assert_ne!(key, GlyphKey::subpixel(font(), 1, 14.0, 0.0));
    }
}
