mod packer;
mod texture;

pub(super) use texture::AtlasTexture;

use packer::ShelfPacker;

pub const INITIAL_ATLAS_SIZE: u32 = 512;
const MAX_MASK_ATLAS_SIZE: u32 = 4096;
const MAX_COLOR_ATLAS_SIZE: u32 = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AtlasKind {
    Mask,
    Color,
}

impl AtlasKind {
    pub fn bytes_per_pixel(self) -> u32 {
        match self {
            AtlasKind::Mask => 1,
            AtlasKind::Color => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasRegion {
    pub kind: AtlasKind,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct Upload {
    pub region: AtlasRegion,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Atlas {
    mask: ShelfPacker,
    color: ShelfPacker,
    uploads: Vec<Upload>,
    largest_side: u32,
    overflowed: bool,
    holds_older_frames: bool,
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new()
    }
}

impl Atlas {
    pub fn new() -> Self {
        Self::within(u32::MAX)
    }

    pub fn limit_to(&mut self, largest_side: u32) {
        *self = Self::within(largest_side);
    }

    fn within(largest_side: u32) -> Self {
        Self {
            mask: packer(MAX_MASK_ATLAS_SIZE.min(largest_side)),
            color: packer(MAX_COLOR_ATLAS_SIZE.min(largest_side)),
            uploads: Vec::new(),
            largest_side,
            overflowed: false,
            holds_older_frames: false,
        }
    }

    pub fn size(&self, kind: AtlasKind) -> u32 {
        self.packer(kind).size()
    }

    pub fn insert(
        &mut self,
        kind: AtlasKind,
        width: u32,
        height: u32,
        data: Vec<u8>,
    ) -> Option<AtlasRegion> {
        let packer = match kind {
            AtlasKind::Mask => &mut self.mask,
            AtlasKind::Color => &mut self.color,
        };
        let Some((x, y)) = packer.allocate(width, height) else {
            self.overflowed = true;
            return None;
        };
        let region = AtlasRegion {
            kind,
            x,
            y,
            width,
            height,
        };
        self.uploads.push(Upload { region, data });
        Some(region)
    }

    pub fn finish_frame(&mut self) -> bool {
        let evict = self.overflowed && self.holds_older_frames;
        self.overflowed = false;
        self.holds_older_frames = true;
        evict
    }

    pub fn clear(&mut self) {
        *self = Self::within(self.largest_side);
    }

    pub fn take_uploads(&mut self) -> Vec<Upload> {
        std::mem::take(&mut self.uploads)
    }

    fn packer(&self, kind: AtlasKind) -> &ShelfPacker {
        match kind {
            AtlasKind::Mask => &self.mask,
            AtlasKind::Color => &self.color,
        }
    }
}

fn packer(max_size: u32) -> ShelfPacker {
    ShelfPacker::new(INITIAL_ATLAS_SIZE.min(max_size), max_size)
}
