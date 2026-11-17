use std::collections::HashMap;
use std::sync::Arc;

use super::{Chunks, Image};

const FIRST_AUTO: u32 = 1 << 31;

#[derive(Debug, Clone)]
pub struct ImageStore {
    images: HashMap<u32, Image>,
    numbers: HashMap<u32, u32>,
    pub pending: Option<Chunks>,
    bytes: usize,
    next_serial: u64,
    next_auto: u32,
    pub virtuals: HashMap<u32, (u16, u16)>,
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
            virtuals: HashMap::new(),
            tallest: 0,
        }
    }
}

impl ImageStore {
    pub const QUOTA: usize = 320 << 20;

    pub fn insert(&mut self, id: u32, width: u32, height: u32, pixels: Vec<u8>) -> &Image {
        self.next_serial += 1;
        self.bytes += pixels.len();
        let image = Image {
            serial: self.next_serial,
            width,
            height,
            pixels: Arc::from(pixels),
        };
        if let Some(old) = self.images.insert(id, image) {
            self.bytes -= old.pixels.len();
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
        let id = self.next_auto;
        self.next_auto = self.next_auto.checked_add(1).unwrap_or(FIRST_AUTO);
        if number != 0 {
            self.numbers.insert(number, id);
        }
        id
    }

    pub fn by_number(&self, number: u32) -> Option<u32> {
        self.numbers.get(&number).copied()
    }

    pub fn remove(&mut self, id: u32) {
        if let Some(old) = self.images.remove(&id) {
            self.bytes -= old.pixels.len();
        }
        self.numbers.retain(|_, image| *image != id);
        self.virtuals.remove(&id);
    }

    pub fn evict_until(&mut self, limit: usize, keep: &dyn Fn(u32) -> bool) {
        while self.bytes > limit {
            let Some(victim) = self
                .oldest(|id| !keep(id))
                .or_else(|| self.oldest(|_| true))
            else {
                return;
            };
            self.remove(victim);
        }
    }

    fn oldest(&self, eligible: impl Fn(u32) -> bool) -> Option<u32> {
        self.images
            .iter()
            .filter(|(id, _)| eligible(**id))
            .min_by_key(|(_, image)| image.serial)
            .map(|(id, _)| *id)
    }

    pub fn clear(&mut self) {
        self.images.clear();
        self.numbers.clear();
        self.pending = None;
        self.bytes = 0;
        self.virtuals.clear();
    }
}
