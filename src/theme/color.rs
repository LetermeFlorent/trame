const DARK_INK: Rgba = Rgba::hex(0x1b1b1b);
const LIGHT_INK: Rgba = Rgba::hex(0xffffff);

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
            ((value >> 16) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        )
    }

    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
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
        let linear = |channel: f32| match channel <= 0.04045 {
            true => channel / 12.92,
            false => ((channel + 0.055) / 1.055).powf(2.4),
        };
        0.2126 * linear(self.r) + 0.7152 * linear(self.g) + 0.0722 * linear(self.b)
    }

    pub fn contrast(self, other: Rgba) -> f32 {
        let (first, second) = (self.luminance(), other.luminance());
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    pub fn ink(self) -> Rgba {
        match self.contrast(DARK_INK) >= self.contrast(LIGHT_INK) {
            true => DARK_INK,
            false => LIGHT_INK,
        }
    }

    pub fn to_rgb8(self) -> [u8; 3] {
        let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        [channel(self.r), channel(self.g), channel(self.b)]
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
