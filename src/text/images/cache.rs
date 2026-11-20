use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use super::ImageKey;
use super::source::{ImageSource, InlineJob, PictureAnswer, PictureJob, SlotTarget};
use super::worker::{ImageWaker, ImageWorker};
use crate::render::{Atlas, AtlasKind, AtlasRegion};
use crate::text::{MAX_CACHED_GLYPHS, insert_bounded};

const MAX_READY_IMAGES: usize = 32;

#[derive(Default)]
pub(in crate::text) struct ImageCache {
    pub(super) regions: HashMap<ImageKey, Option<AtlasRegion>>,
    pub(super) hover: Option<(u64, AtlasRegion)>,
    pub(super) ready: HashMap<ImageKey, Vec<u8>>,
    ready_order: VecDeque<ImageKey>,
    pub(super) pending: HashSet<ImageKey>,
    pub(super) picture: Option<PictureAnswer>,
    pub(super) preview: Option<PictureAnswer>,
    pub(super) worker: ImageWorker,
}

impl ImageCache {
    pub(in crate::text) fn clear_regions(&mut self) {
        self.regions.clear();
        self.hover = None;
    }

    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub(super) fn cache_region(
        &mut self,
        atlas: &mut Atlas,
        key: ImageKey,
        data: Vec<u8>,
    ) -> Option<AtlasRegion> {
        let region = atlas.insert(AtlasKind::Color, key.width, key.height, data)?;
        if self.regions.len() >= MAX_CACHED_GLYPHS {
            atlas.request_reset();
        }
        self.regions.insert(key, Some(region));
        Some(region)
    }

    pub(super) fn record_ready(&mut self, key: ImageKey, pixels: Vec<u8>) {
        if let Some(slot) = self.ready.get_mut(&key) {
            *slot = pixels;
            return;
        }
        while self.ready.len() >= MAX_READY_IMAGES {
            let Some(oldest) = self.ready_order.pop_front() else {
                self.ready.clear();
                break;
            };
            self.ready.remove(&oldest);
        }
        self.ready_order.push_back(key);
        self.ready.insert(key, pixels);
    }

    pub(super) fn take_ready(&mut self, key: ImageKey) {
        self.ready.remove(&key);
        self.ready_order.retain(|queued| *queued != key);
    }

    pub(super) fn fail(&mut self, key: ImageKey) {
        insert_bounded(&mut self.regions, key, None);
    }

    pub(super) fn enqueue_inline(&mut self, key: ImageKey, limit: (u32, u32), bytes: &Arc<[u8]>) {
        if self.pending.insert(key) {
            self.worker.push_inline(InlineJob {
                key,
                limit,
                bytes: Arc::clone(bytes),
            });
        }
    }

    pub(super) fn set_waker(&mut self, wake: ImageWaker) {
        self.worker.set_waker(wake);
    }

    pub(super) fn request_slot(
        &mut self,
        target: SlotTarget,
        job: u64,
        limit: (u32, u32),
        source: ImageSource,
    ) {
        match target {
            SlotTarget::Picture => self.picture = None,
            SlotTarget::Preview => self.preview = None,
        }
        self.worker
            .set_slot(target, PictureJob { job, limit, source });
    }
}
