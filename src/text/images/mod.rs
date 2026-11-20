mod cache;
mod decode;
mod decoded;
mod slots;
mod source;
#[cfg(test)]
mod tests;
mod worker;

use std::sync::Arc;

use super::TextSystem;
use crate::render::AtlasRegion;

pub(super) use cache::ImageCache;
pub use source::{DecodedPicture, ImageSource};
pub use worker::ImageWaker;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageKey {
    pub id: u64,
    pub width: u32,
    pub height: u32,
}

impl TextSystem {
    pub fn max_image_side(&self) -> u32 {
        self.atlas.max_image_side()
    }

    pub fn image(
        &mut self,
        key: ImageKey,
        pixels: impl FnOnce() -> Vec<u8>,
    ) -> Option<AtlasRegion> {
        self.try_image(key, || Some(pixels()))
    }

    pub fn try_image(
        &mut self,
        key: ImageKey,
        pixels: impl FnOnce() -> Option<Vec<u8>>,
    ) -> Option<AtlasRegion> {
        if key.width == 0 || key.height == 0 {
            return None;
        }
        if let Some(found) = self.images.regions.get(&key) {
            return *found;
        }
        match pixels() {
            Some(data) => self.images.cache_region(&mut self.atlas, key, data),
            None => {
                self.images.fail(key);
                None
            }
        }
    }

    pub fn queued_image(
        &mut self,
        key: ImageKey,
        limit: (u32, u32),
        bytes: &Arc<[u8]>,
    ) -> Option<AtlasRegion> {
        if key.width == 0 || key.height == 0 || limit.0 == 0 || limit.1 == 0 {
            return None;
        }
        if let Some(found) = self.images.regions.get(&key) {
            return *found;
        }
        if let Some(data) = self.images.ready.get(&key) {
            let region = self
                .images
                .cache_region(&mut self.atlas, key, data.clone())?;
            self.images.take_ready(key);
            return Some(region);
        }
        self.images.enqueue_inline(key, limit, bytes);
        None
    }

    pub fn set_image_waker(&mut self, wake: ImageWaker) {
        self.images.set_waker(wake);
    }

    pub fn receive_images(&mut self) -> bool {
        let (inline, picture, preview) = self.images.worker.drain();
        let any = !inline.is_empty() || picture.is_some() || preview.is_some();
        for answer in inline {
            self.images.pending.remove(&answer.key);
            match answer.pixels {
                Some(pixels) => self.images.record_ready(answer.key, pixels),
                None => self.images.fail(answer.key),
            }
        }
        self.images.picture = picture.or(self.images.picture.take());
        self.images.preview = preview.or(self.images.preview.take());
        any
    }

    pub fn has_pending_images(&self) -> bool {
        !self.images.pending.is_empty()
    }
}
