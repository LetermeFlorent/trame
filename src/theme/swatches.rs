use serde::{Deserialize, Serialize};

use super::{Appearance, Theme};
use crate::color::Rgba;

const TINT_DARK: f32 = 0.16;
const TINT_LIGHT: f32 = 0.12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Swatch {
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Blue,
    Purple,
    Pink,
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
        Rgba::hex(table[swatch as usize])
    }

    pub fn swatch_tint(&self, swatch: Swatch) -> Rgba {
        let strength = match self.appearance {
            Appearance::Dark => TINT_DARK,
            Appearance::Light => TINT_LIGHT,
        };
        self.swatch(swatch).with_alpha(strength)
    }
}
