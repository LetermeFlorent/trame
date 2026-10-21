use super::{FontId, FontKey, FontLibrary, REGULAR};

const STYLE_MISMATCH: u32 = 10_000;

#[derive(Debug, Clone, Default)]
pub struct Family {
    faces: Vec<FontId>,
}

impl Family {
    pub fn new(faces: Vec<FontId>) -> Self {
        Self { faces }
    }

    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    pub fn primary(&self, library: &FontLibrary) -> Option<FontId> {
        self.resolve(library, REGULAR, false).map(|key| key.id)
    }

    pub fn resolve(&self, library: &FontLibrary, weight: u16, italic: bool) -> Option<FontKey> {
        let id = self
            .faces
            .iter()
            .copied()
            .min_by_key(|id| distance(library, *id, weight, italic))?;
        Some(FontKey::synthesize(library, id, weight, italic))
    }
}

fn distance(library: &FontLibrary, id: FontId, weight: u16, italic: bool) -> u32 {
    let face = library.face(id);
    let effective = face.weight_variation(weight).unwrap_or(face.weight());
    let style = if face.is_italic() == italic {
        0
    } else {
        STYLE_MISMATCH
    };
    style + u32::from(effective.abs_diff(weight))
}
