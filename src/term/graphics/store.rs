use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use super::{Chunks, Image};

const FIRST_AUTO: u32 = 1 << 31;
const IMAGE_OVERHEAD: usize = 128;

pub type Virtuals = BTreeMap<(u32, u32), (u16, u16)>;

#[derive(Debug, Clone)]
pub struct ImageStore {
    pub(super) images: HashMap<u32, Image>,
    pub(super) numbers: HashMap<u32, u32>,
    pub pending: Option<Chunks>,
    pub(super) bytes: usize,
    next_serial: u64,
    next_auto: u32,
    pub virtuals: Virtuals,
    tallest: u16,
}

impl Default for ImageStore {
    fn default() -> Self {
        Self {
            images: HashMap::new(),
            numbers: HashMap::new(),
            pending: None,
            bytes: 0,
            next_serial: 0,
            next_auto: FIRST_AUTO,
            virtuals: Virtuals::new(),
            tallest: 0,
        }
    }
}

pub(super) fn cost(image: &Image) -> usize {
    image.pixels.len() + IMAGE_OVERHEAD
}

impl ImageStore {
    pub const QUOTA: usize = 320 << 20;
    pub const OVERHEAD: usize = IMAGE_OVERHEAD;

    pub fn insert(&mut self, id: u32, width: u32, height: u32, pixels: Vec<u8>) -> &Image {
        self.next_serial += 1;
        let image = Image {
            serial: self.next_serial,
            width,
            height,
            pixels: Arc::from(pixels),
        };
        self.bytes += cost(&image);
        if let Some(old) = self.images.insert(id, image) {
            self.bytes -= cost(&old);
        }
        &self.images[&id]
    }

    pub fn get(&self, id: u32) -> Option<&Image> {
        self.images.get(&id)
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    pub fn ids(&self) -> Vec<u32> {
        self.images.keys().copied().collect()
    }

    pub fn tallest(&self) -> u16 {
        self.tallest
    }

    pub fn note_rows(&mut self, rows: u16) {
        self.tallest = self.tallest.max(rows);
    }

    pub fn assign(&mut self, number: u32) -> u32 {
        let mut id = self.next_auto;
        while self.images.contains_key(&id) {
            id = id.checked_add(1).unwrap_or(FIRST_AUTO);
        }
        self.next_auto = id.checked_add(1).unwrap_or(FIRST_AUTO);
        if number != 0 {
            self.numbers.insert(number, id);
        }
        id
    }

    pub fn by_number(&self, number: u32) -> Option<u32> {
        self.numbers.get(&number).copied()
    }

    pub fn virtual_span(&self, image: u32) -> Option<(u16, u16)> {
        self.virtuals
            .range((image, 0)..=(image, u32::MAX))
            .next()
            .map(|(_, &span)| span)
    }

    pub fn remove(&mut self, id: u32) {
        self.remove_all(&[id]);
    }
    pub fn clear(&mut self) {
        self.images.clear();
        self.numbers.clear();
        self.pending = None;
        self.bytes = 0;
        self.virtuals.clear();
    }
}
