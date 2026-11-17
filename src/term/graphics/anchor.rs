use super::Crop;

#[derive(Debug, Clone, PartialEq)]
pub struct Anchor {
    pub col: u16,
    pub image: u32,
    pub placement: u32,
    pub cols: u16,
    pub rows: u16,
    pub crop: Crop,
    pub offset: (u16, u16),
    pub z: i32,
}

impl Anchor {
    pub fn covers(&self, at: isize, row: isize, col: usize) -> bool {
        let first = usize::from(self.col);
        (at..at + self.rows as isize).contains(&row)
            && (first..first + usize::from(self.cols)).contains(&col)
    }

    pub fn same_placement(&self, other: &Anchor) -> bool {
        self.placement != 0 && (self.image, self.placement) == (other.image, other.placement)
    }
}
