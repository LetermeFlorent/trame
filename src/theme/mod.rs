mod ansi;
mod color;
mod emblems;
mod hsv;
mod letters;
mod palettes;
mod swatches;
pub mod system;

pub use color::Rgba;
pub use emblems::Emblem;
pub use hsv::{Hsv, hex_code, parse_hex};
pub use letters::{Letters, MAX_LETTERS};
pub use swatches::Swatch;

use crate::term::{Color, ReportedColors};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Appearance {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub appearance: Appearance,
    pub chrome: Rgba,
    pub tab_bar: Rgba,
    pub terminal: Rgba,
    pub foreground: Rgba,
    pub border: Rgba,
    pub border_strong: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub text_faint: Rgba,
    pub accent: Rgba,
    pub hover: Rgba,
    pub pressed: Rgba,
    pub selection: Rgba,
    pub drop_target: Rgba,
    pub shadow: Rgba,
    pub ansi: [Rgba; 16],
}

impl Theme {
    pub fn new(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Dark => palettes::dark(),
            Appearance::Light => palettes::light(),
        }
    }

    pub fn muted_text(&self, glow: f32) -> Rgba {
        self.text_muted.mix(self.text, glow)
    }

    pub fn terminal_color(&self, color: Color) -> Rgba {
        match color {
            Color::Foreground => self.foreground,
            Color::Background => self.terminal,
            Color::Indexed(index) => self.indexed(index),
            Color::Rgb(red, green, blue) => Rgba::rgb8(red, green, blue),
        }
    }

    pub fn indexed(&self, index: u8) -> Rgba {
        self.ansi
            .get(usize::from(index))
            .copied()
            .unwrap_or_else(|| ansi::extended(index))
    }

    pub fn reported_colors(&self) -> ReportedColors {
        ReportedColors {
            foreground: self.foreground.to_rgb8(),
            background: self.terminal.to_rgb8(),
            cursor: self.accent.to_rgb8(),
        }
    }
}
