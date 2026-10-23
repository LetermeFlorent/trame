use super::TextSystem;
use crate::render::{AtlasKind, AtlasRegion};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageKey {
    pub id: u64,
    pub size: u32,
}

impl TextSystem {
    pub fn image(
        &mut self,
        key: ImageKey,
        pixels: impl FnOnce() -> Vec<u8>,
    ) -> Option<AtlasRegion> {
        if let Some(found) = self.images.get(&key) {
            return Some(*found);
        }
        let region = self
            .atlas
            .insert(AtlasKind::Color, key.size, key.size, pixels())?;
        self.images.insert(key, region);
        Some(region)
    }
}
