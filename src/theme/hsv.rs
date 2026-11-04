pub const RGB_MASK: u32 = 0x00ff_ffff;

const HEX_DIGITS: usize = 6;
const SHORT_DIGITS: usize = 3;
const SECTORS: f32 = 6.0;
const CHANNEL: f32 = u8::MAX as f32;
const BYTE_MASK: u32 = 0xFF;
const RED_SHIFT: u32 = 16;
const GREEN_SHIFT: u32 = 8;
const HEX_RADIX: u32 = 16;
const GREEN_SECTOR_OFFSET: f32 = 2.0;
const BLUE_SECTOR_OFFSET: f32 = 4.0;
const RAMP_WIDTH: f32 = 4.0;
const RED_RAMP_OFFSET: f32 = 5.0;
const GREEN_RAMP_OFFSET: f32 = 3.0;
const BLUE_RAMP_OFFSET: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsv {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

impl Hsv {
    pub fn from_rgb(rgb: u32) -> Self {
        let [r, g, b] = channels(rgb);
        let max = r.max(g).max(b);
        let delta = max - r.min(g).min(b);
        let sector = match () {
            _ if delta <= 0.0 => 0.0,
            _ if max == r => ((g - b) / delta).rem_euclid(SECTORS),
            _ if max == g => (b - r) / delta + GREEN_SECTOR_OFFSET,
            _ => (r - g) / delta + BLUE_SECTOR_OFFSET,
        };
        let saturation = match max > 0.0 {
            true => delta / max,
            false => 0.0,
        };
        Self {
            hue: sector / SECTORS,
            saturation,
            value: max,
        }
    }

    pub fn to_rgb(self) -> u32 {
        let hue = self.hue.clamp(0.0, 1.0) * SECTORS;
        let saturation = self.saturation.clamp(0.0, 1.0);
        let value = self.value.clamp(0.0, 1.0);
        let channel = |offset: f32| {
            let k = (offset + hue).rem_euclid(SECTORS);
            let ramp = k.min(RAMP_WIDTH - k).clamp(0.0, 1.0);
            let level = value - value * saturation * ramp;
            (level * CHANNEL).round() as u32
        };
        channel(RED_RAMP_OFFSET) << RED_SHIFT
            | channel(GREEN_RAMP_OFFSET) << GREEN_SHIFT
            | channel(BLUE_RAMP_OFFSET)
    }

    pub fn pure_hue(self) -> u32 {
        Self {
            saturation: 1.0,
            value: 1.0,
            ..self
        }
        .to_rgb()
    }
}

fn channels(rgb: u32) -> [f32; 3] {
    [RED_SHIFT, GREEN_SHIFT, 0].map(|shift| ((rgb >> shift) & BYTE_MASK) as f32 / CHANNEL)
}

pub fn hex_code(rgb: u32) -> String {
    format!("#{:06X}", rgb & RGB_MASK)
}

pub fn parse_hex(text: &str) -> Option<u32> {
    let digits = text.trim().trim_start_matches('#');
    if !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
        return None;
    }
    match digits.len() {
        HEX_DIGITS => u32::from_str_radix(digits, HEX_RADIX).ok(),
        SHORT_DIGITS => {
            let doubled: String = digits.chars().flat_map(|digit| [digit, digit]).collect();
            u32::from_str_radix(&doubled, HEX_RADIX).ok()
        }
        _ => None,
    }
}
