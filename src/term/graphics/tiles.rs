use super::{Anchor, Image};
use crate::term::screen::Screen;

const MAX_LOOKBACK: u64 = 1024;
const UNDER_BACKGROUND: i32 = i32::MIN / 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    UnderBackground,
    UnderText,
    OverText,
}

impl Layer {
    pub fn of(z: i32) -> Self {
        if z < UNDER_BACKGROUND {
            Self::UnderBackground
        } else if z < 0 {
            Self::UnderText
        } else {
            Self::OverText
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageTile {
    pub image: Image,
    pub uv: [f32; 4],
    pub row: isize,
    pub col: usize,
    pub cols: f32,
    pub rows: f32,
    pub offset: (u16, u16),
    pub layer: Layer,
}

impl ImageTile {
    fn new(image: &Image, anchor: &Anchor, row: isize) -> Self {
        let (width, height) = (image.width.max(1) as f32, image.height.max(1) as f32);
        let crop = anchor.crop;
        Self {
            image: image.clone(),
            uv: [
                crop.x as f32 / width,
                crop.y as f32 / height,
                (crop.x + crop.w) as f32 / width,
                (crop.y + crop.h) as f32 / height,
            ],
            row,
            col: usize::from(anchor.col),
            cols: f32::from(anchor.cols),
            rows: f32::from(anchor.rows),
            offset: anchor.offset,
            layer: Layer::of(anchor.z),
        }
    }
}

impl Screen {
    pub fn image_tiles(&self) -> Vec<ImageTile> {
        let grid = self.grid();
        self.image_tiles_at(grid.first_visible_line(), grid.rows())
    }

    pub fn image_tiles_at(&self, first_line: u64, rows: usize) -> Vec<ImageTile> {
        if self.graphics.is_empty() {
            return Vec::new();
        }
        let grid = self.grid();
        let lookback = u64::from(self.graphics.tallest().saturating_sub(1)).min(MAX_LOOKBACK);
        let mut found: Vec<(i32, ImageTile)> = Vec::new();
        for line in first_line.saturating_sub(lookback)..first_line + rows as u64 {
            let Some(row) = grid.line(line) else {
                continue;
            };
            let at = line as isize - first_line as isize;
            for anchor in row.anchors() {
                let shown = at + anchor.rows as isize > 0;
                if let Some(image) = self.graphics.get(anchor.image).filter(|_| shown) {
                    found.push((anchor.z, ImageTile::new(image, anchor, at)));
                }
            }
        }
        found.sort_by_key(|(z, _)| *z);
        found.into_iter().map(|(_, tile)| tile).collect()
    }
}
