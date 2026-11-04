mod discovery;
mod face;
mod fallback;
mod family;
mod library;

use std::fmt;

pub use discovery::system_fonts;
pub use face::{Face, WEIGHT_AXIS};
pub use family::Family;
pub use library::FontLibrary;

pub const REGULAR: u16 = 400;
pub const SEMIBOLD: u16 = 600;
pub const BOLD: u16 = 700;

#[derive(Debug, Clone, Copy)]
pub struct MissingFont;

impl fmt::Display for MissingFont {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("no usable monospace font was found on this system")
    }
}

impl std::error::Error for MissingFont {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontId(u16);

impl FontId {
    #[cfg(test)]
    pub fn first() -> Self {
        Self(0)
    }

    fn index(self) -> usize {
        usize::from(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontKey {
    pub id: FontId,
    pub weight: Option<u16>,
    pub embolden: bool,
    pub skew: bool,
}

impl FontKey {
    pub fn synthesize(library: &FontLibrary, id: FontId, weight: u16, italic: bool) -> Self {
        let face = library.face(id);
        let effective = face.effective_weight(weight);
        Self {
            id,
            weight: face.weight_variation(weight),
            embolden: weight >= SEMIBOLD && effective < SEMIBOLD,
            skew: italic && !face.is_italic(),
        }
    }
}
