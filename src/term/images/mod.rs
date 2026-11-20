mod inline;
mod place;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::term::screen::Screen;

pub const MAX_IMAGE_STORE_BYTES: usize = 32 * 1024 * 1024;
const MAX_IMAGE_ID: u32 = 1 << 24;
const IMAGE_TILE_BASE: u32 = 0x10_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredImage {
    pub bytes: Arc<[u8]>,
    pub width: u32,
    pub height: u32,
    pub rows: u16,
    pub cols: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct ImageStore {
    entries: HashMap<u32, Arc<StoredImage>>,
    order: VecDeque<u32>,
    total_bytes: usize,
    next_id: u32,
}

impl Default for ImageStore {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            total_bytes: 0,
            next_id: 1,
        }
    }
}

impl ImageStore {
    pub(crate) fn get(&self, id: u32) -> Option<&Arc<StoredImage>> {
        self.entries.get(&id)
    }

    pub(crate) fn bytes(&self) -> usize {
        self.total_bytes
    }

    pub(crate) fn keep_across_reset(&self) -> Self {
        Self {
            next_id: self.next_id,
            ..Self::default()
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn prune(&mut self, used: &HashSet<u32>) {
        self.entries.retain(|id, image| {
            let keep = used.contains(id);
            if !keep {
                self.total_bytes = self.total_bytes.saturating_sub(image.bytes.len());
            }
            keep
        });
        self.order.retain(|id| self.entries.contains_key(id));
    }

    pub(crate) fn insert(
        &mut self,
        image: StoredImage,
        mark_used: impl FnOnce(&mut HashSet<u32>),
    ) -> u32 {
        let needed = image.bytes.len();
        if self.total_bytes + needed > MAX_IMAGE_STORE_BYTES {
            let mut used = HashSet::new();
            mark_used(&mut used);
            self.prune(&used);
            while self.total_bytes + needed > MAX_IMAGE_STORE_BYTES {
                let Some(oldest) = self.order.pop_front() else {
                    break;
                };
                if let Some(removed) = self.entries.remove(&oldest) {
                    self.total_bytes = self.total_bytes.saturating_sub(removed.bytes.len());
                }
            }
        }
        let id = self.allocate_id();
        self.total_bytes += needed;
        self.entries.insert(id, Arc::new(image));
        self.order.push_back(id);
        id
    }

    fn allocate_id(&mut self) -> u32 {
        loop {
            let id = self.next_id;
            self.next_id = if id + 1 >= MAX_IMAGE_ID { 1 } else { id + 1 };
            if !self.entries.contains_key(&id) {
                return id;
            }
        }
    }
}

pub(crate) fn tile_char(row: usize, col: usize) -> char {
    char::from_u32(IMAGE_TILE_BASE + (row as u32) * 256 + (col as u32)).unwrap_or(' ')
}

impl Screen {
    pub fn image(&self, id: u32) -> Option<&Arc<StoredImage>> {
        self.images.get(id)
    }

    pub fn image_at(&self, row: usize, col: usize) -> Option<(u32, Arc<StoredImage>)> {
        if row >= self.rows() {
            return None;
        }
        let id = self.grid().visible_row(row).cell(col).image_id()?;
        Some((id, self.images.get(id)?.clone()))
    }

    pub fn image_bytes(&self) -> usize {
        self.images.bytes()
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub(crate) fn collect_images(&mut self) {
        let mut used = HashSet::new();
        self.primary.mark_images(&mut used);
        self.alternate.mark_images(&mut used);
        self.images.prune(&used);
    }

    pub(crate) fn insert_image(&mut self, image: StoredImage) -> u32 {
        let primary = &self.primary;
        let alternate = &self.alternate;
        self.images.insert(image, |used| {
            primary.mark_images(used);
            alternate.mark_images(used);
        })
    }
}
