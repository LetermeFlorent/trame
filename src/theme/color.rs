const DARK_INK: Rgba = Rgba::hex(0x1b1b1b);
const LIGHT_INK: Rgba = Rgba::hex(0xffffff);

const BYTE_MASK: u32 = 0xFF;
const RED_SHIFT: u32 = 16;
const GREEN_SHIFT: u32 = 8;
const MAX_CHANNEL: f32 = u8::MAX as f32;
const SRGB_LINEAR_LIMIT: f32 = 0.04045;
const SRGB_LINEAR_SLOPE: f32 = 12.92;
const SRGB_OFFSET: f32 = 0.055;
const SRGB_GAMMA: f32 = 2.4;
const LUMA_RED: f32 = 0.2126;
const LUMA_GREEN: f32 = 0.7152;
const LUMA_BLUE: f32 = 0.0722;
const CONTRAST_FLARE: f32 = 0.05;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn hex(value: u32) -> Self {
        Self::rgb8(
            ((value >> RED_SHIFT) & BYTE_MASK) as u8,
            ((value >> GREEN_SHIFT) & BYTE_MASK) as u8,
            (value & BYTE_MASK) as u8,
        )
    }

    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::new(
            r as f32 / MAX_CHANNEL,
            g as f32 / MAX_CHANNEL,
            b as f32 / MAX_CHANNEL,
            1.0,
        )
    }

    pub const fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    pub fn faded(self, factor: f32) -> Self {
        self.with_alpha(self.a * factor)
    }

    pub fn mix(self, other: Rgba, amount: f32) -> Self {
        let lerp = |from: f32, to: f32| from + (to - from) * amount;
        Self::new(
            lerp(self.r, other.r),
            lerp(self.g, other.g),
            lerp(self.b, other.b),
            lerp(self.a, other.a),
        )
    }

    pub fn luminance(self) -> f32 {
        let linear = |channel: f32| match channel <= SRGB_LINEAR_LIMIT {
            true => channel / SRGB_LINEAR_SLOPE,
            false => ((channel + SRGB_OFFSET) / (1.0 + SRGB_OFFSET)).powf(SRGB_GAMMA),
        };
        LUMA_RED * linear(self.r) + LUMA_GREEN * linear(self.g) + LUMA_BLUE * linear(self.b)
    }

    pub fn contrast(self, other: Rgba) -> f32 {
        let (first, second) = (self.luminance(), other.luminance());
        (first.max(second) + CONTRAST_FLARE) / (first.min(second) + CONTRAST_FLARE)
    }

    pub fn ink(self) -> Rgba {
        match self.contrast(DARK_INK) >= self.contrast(LIGHT_INK) {
            true => DARK_INK,
            false => LIGHT_INK,
        }
    }

    pub fn to_rgb8(self) -> [u8; 3] {
        let channel = |value: f32| (value.clamp(0.0, 1.0) * MAX_CHANNEL).round() as u8;
        [channel(self.r), channel(self.g), channel(self.b)]
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
