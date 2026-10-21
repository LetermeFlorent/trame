use super::font::Face;

pub const DEFAULT_LINE_HEIGHT: f32 = 1.2;

const WIDTH_REFERENCE: char = '0';
const FALLBACK_ADVANCE: f32 = 0.6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellMetrics {
    pub font_size: f32,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub underline_position: f32,
    pub underline_thickness: f32,
    pub strike_position: f32,
    pub strike_thickness: f32,
}

impl CellMetrics {
    pub fn measure(face: &Face, font_size: f32, line_height: f32) -> Self {
        let metrics = face.metrics(font_size);
        let advance = match face.advance(face.glyph(WIDTH_REFERENCE), font_size) {
            advance if advance > 0.0 => advance,
            _ => font_size * FALLBACK_ADVANCE,
        };
        let natural = metrics.ascent + metrics.descent;
        let height = (natural * line_height.max(1.0)).round().max(1.0);
        let baseline = ((height - natural) / 2.0 + metrics.ascent).round();
        let thickness = metrics.stroke_size.round().max(1.0);
        let underline = (baseline - metrics.underline_offset)
            .round()
            .max(baseline + 1.0)
            .min(height - thickness);
        let strike_offset = if metrics.strikeout_offset > 0.0 {
            metrics.strikeout_offset
        } else {
            metrics.x_height / 2.0
        };
        Self {
            font_size,
            width: advance.round().max(1.0),
            height,
            baseline,
            underline_position: underline,
            underline_thickness: thickness,
            strike_position: (baseline - strike_offset).round(),
            strike_thickness: thickness,
        }
    }
}
