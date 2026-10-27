use std::sync::Arc;

use swash::{CacheKey, FontRef, Metrics, Style, Tag, tag_from_bytes};

pub type FontData = Arc<dyn AsRef<[u8]> + Send + Sync>;

pub const WEIGHT_AXIS: Tag = tag_from_bytes(b"wght");
const CAP_HEIGHT_TO_ASCENT: f32 = 0.72;

pub struct Face {
    data: FontData,
    offset: u32,
    key: CacheKey,
    weight: u16,
    italic: bool,
    weight_axis: Option<(f32, f32)>,
}

impl Face {
    pub fn new(data: FontData, index: u32) -> Option<Self> {
        let font = FontRef::from_index((*data).as_ref(), index as usize)?;
        let (_, weight, style) = font.attributes().parts();
        let weight_axis = font
            .variations()
            .find(|axis| axis.tag() == WEIGHT_AXIS)
            .map(|axis| (axis.min_value(), axis.max_value()));
        Some(Self {
            offset: font.offset,
            key: font.key,
            weight: weight.0,
            italic: style != Style::Normal,
            weight_axis,
            data,
        })
    }

    pub fn font(&self) -> FontRef<'_> {
        FontRef {
            data: (*self.data).as_ref(),
            offset: self.offset,
            key: self.key,
        }
    }

    pub fn glyph(&self, ch: char) -> u16 {
        self.font().charmap().map(ch)
    }

    pub fn covers(&self, ch: char) -> bool {
        self.glyph(ch) != 0
    }

    pub fn weight(&self) -> u16 {
        self.weight
    }

    pub fn is_italic(&self) -> bool {
        self.italic
    }

    pub fn weight_variation(&self, weight: u16) -> Option<u16> {
        let (min, max) = self.weight_axis?;
        Some(f32::from(weight).clamp(min, max).round() as u16)
    }

    pub fn effective_weight(&self, weight: u16) -> u16 {
        self.weight_variation(weight).unwrap_or(self.weight)
    }

    pub fn metrics(&self, size: f32) -> Metrics {
        let mut metrics = self.font().metrics(&[]).scale(size);
        if metrics.cap_height <= 0.0 {
            metrics.cap_height = metrics.ascent * CAP_HEIGHT_TO_ASCENT;
        }
        metrics
    }

    pub fn advance(&self, glyph: u16, size: f32) -> f32 {
        self.font()
            .glyph_metrics(&[])
            .scale(size)
            .advance_width(glyph)
    }
}
