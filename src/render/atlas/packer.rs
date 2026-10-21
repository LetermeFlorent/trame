const PADDING: u32 = 1;

#[derive(Debug)]
struct Shelf {
    y: u32,
    height: u32,
    cursor: u32,
}

#[derive(Debug)]
pub(super) struct ShelfPacker {
    size: u32,
    max_size: u32,
    shelves: Vec<Shelf>,
    next_y: u32,
}

impl ShelfPacker {
    pub(super) fn new(size: u32, max_size: u32) -> Self {
        Self {
            size,
            max_size,
            shelves: Vec::new(),
            next_y: 0,
        }
    }

    pub(super) fn size(&self) -> u32 {
        self.size
    }

    pub(super) fn clear(&mut self) {
        self.shelves.clear();
        self.next_y = 0;
    }

    pub(super) fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let width = width + PADDING;
        let height = height + PADDING;
        if width > self.max_size || height > self.max_size {
            return None;
        }
        loop {
            if let Some(position) = self.place(width, height) {
                return Some(position);
            }
            if self.size >= self.max_size {
                return None;
            }
            self.size *= 2;
        }
    }

    fn place(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let size = self.size;
        let best = self
            .shelves
            .iter_mut()
            .filter(|shelf| shelf.height >= height && shelf.height <= height * 2)
            .filter(|shelf| shelf.cursor + width <= size)
            .min_by_key(|shelf| shelf.height);
        if let Some(shelf) = best {
            let position = (shelf.cursor, shelf.y);
            shelf.cursor += width;
            return Some(position);
        }
        if self.next_y + height > size || width > size {
            return None;
        }
        let y = self.next_y;
        self.next_y += height;
        self.shelves.push(Shelf {
            y,
            height,
            cursor: width,
        });
        Some((0, y))
    }
}

#[cfg(test)]
mod tests {
    use super::ShelfPacker;

    #[test]
    fn packs_side_by_side_then_grows() {
        let mut packer = ShelfPacker::new(16, 64);
        assert_eq!(packer.allocate(7, 7), Some((0, 0)));
        assert_eq!(packer.allocate(7, 7), Some((8, 0)));
        assert_eq!(packer.allocate(7, 7), Some((0, 8)));
        assert_eq!(packer.allocate(7, 7), Some((8, 8)));
        assert_eq!(packer.allocate(7, 7), Some((16, 0)));
        assert_eq!(packer.size(), 32);
    }

    #[test]
    fn refuses_when_full_at_max_size() {
        let mut packer = ShelfPacker::new(8, 8);
        assert!(packer.allocate(7, 7).is_some());
        assert_eq!(packer.allocate(7, 7), None);
        assert_eq!(packer.allocate(100, 1), None);
    }
}
