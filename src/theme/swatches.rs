use serde::{Deserialize, Serialize};

use super::hsv::{RGB_MASK, hex_code, parse_hex};
use super::{Appearance, Theme};
use crate::theme::Rgba;

const TINT_DARK: f32 = 0.16;
const TINT_LIGHT: f32 = 0.12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Swatch {
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Blue,
    Purple,
    Pink,
    Custom(u32),
}

impl Swatch {
    pub const ALL: [Swatch; 8] = [
        Swatch::Red,
        Swatch::Orange,
        Swatch::Yellow,
        Swatch::Green,
        Swatch::Teal,
        Swatch::Blue,
        Swatch::Purple,
        Swatch::Pink,
    ];

    const NAMES: [&'static str; 8] = [
        "red", "orange", "yellow", "green", "teal", "blue", "purple", "pink",
    ];

    pub fn custom(rgb: u32) -> Swatch {
        Swatch::Custom(rgb & RGB_MASK)
    }

    pub fn is_custom(self) -> bool {
        matches!(self, Swatch::Custom(_))
    }

    const fn table_slot(self) -> Result<usize, u32> {
        match self {
            Swatch::Red => Ok(0),
            Swatch::Orange => Ok(1),
            Swatch::Yellow => Ok(2),
            Swatch::Green => Ok(3),
            Swatch::Teal => Ok(4),
            Swatch::Blue => Ok(5),
            Swatch::Purple => Ok(6),
            Swatch::Pink => Ok(7),
            Swatch::Custom(rgb) => Err(rgb),
        }
    }

    pub fn parse(text: &str) -> Option<Swatch> {
        match Self::NAMES.iter().position(|name| *name == text) {
            Some(index) => Some(Self::ALL[index]),
            None => parse_hex(text).map(Swatch::custom),
        }
    }
}

impl From<Swatch> for String {
    fn from(swatch: Swatch) -> String {
        match swatch.table_slot() {
            Ok(index) => Swatch::NAMES[index].to_owned(),
            Err(rgb) => hex_code(rgb),
        }
    }
}

impl TryFrom<String> for Swatch {
    type Error = String;

    fn try_from(text: String) -> Result<Swatch, String> {
        Swatch::parse(&text).ok_or_else(|| format!("unknown colour {text}"))
    }
}

const DARK: [u32; 8] = [
    0xe5757d, 0xe8a05c, 0xe3c46f, 0x95c76f, 0x5bbcc8, 0x74ade8, 0xb98bd9, 0xe67fb4,
];

const LIGHT: [u32; 8] = [
    0xc4453f, 0xb85f0a, 0x8f6d00, 0x3d8a3a, 0x1d7f8f, 0x2f6fcf, 0x9a42b8, 0xb83a80,
];

impl Theme {
    pub fn swatch(&self, swatch: Swatch) -> Rgba {
        let table = match self.appearance {
            Appearance::Dark => &DARK,
            Appearance::Light => &LIGHT,
        };
        let rgb = swatch.table_slot().map(|index| table[index]);
        Rgba::hex(rgb.unwrap_or_else(|custom| custom))
    }

    pub fn swatch_tint(&self, swatch: Swatch) -> Rgba {
        let strength = match self.appearance {
            Appearance::Dark => TINT_DARK,
            Appearance::Light => TINT_LIGHT,
        };
        self.swatch(swatch).with_alpha(strength)
    }
}
