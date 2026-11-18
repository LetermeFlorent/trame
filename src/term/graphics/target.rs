use super::{Anchor, Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Visible,
    Image(u32, u32),
    Ids(u32, u32),
    Cell(isize, usize),
    Column(usize),
    Row(isize),
    Layer(i32),
}

impl Target {
    pub fn of(command: &Command, id: u32, cursor: (usize, usize)) -> Option<Self> {
        let (x, y) = (command.crop.x as usize, command.crop.y as isize);
        Some(match command.delete.to_ascii_lowercase() {
            b'a' => Self::Visible,
            b'i' | b'n' if id != 0 => Self::Image(id, command.placement),
            b'r' => Self::Ids(command.crop.x, command.crop.y),
            b'p' => Self::Cell(y - 1, x.checked_sub(1)?),
            b'c' => Self::Cell(cursor.0 as isize, cursor.1),
            b'x' => Self::Column(x.checked_sub(1)?),
            b'y' => Self::Row(y - 1),
            b'z' => Self::Layer(command.z),
            _ => return None,
        })
    }

    pub fn reaches_history(&self) -> bool {
        matches!(self, Self::Image(..) | Self::Ids(..))
    }

    pub fn names(&self, image: u32) -> bool {
        match *self {
            Self::Image(id, _) => id == image,
            Self::Ids(first, last) => (first..=last).contains(&image),
            _ => false,
        }
    }

    pub fn names_placement(&self, image: u32, placement: u32) -> bool {
        match *self {
            Self::Image(id, wanted) => id == image && (wanted == 0 || wanted == placement),
            _ => self.names(image),
        }
    }

    pub fn hits(&self, at: isize, anchor: &Anchor) -> bool {
        match *self {
            Self::Visible => true,
            Self::Image(..) | Self::Ids(..) => self.names_placement(anchor.image, anchor.placement),
            Self::Cell(row, col) => anchor.covers(at, row, col),
            Self::Column(col) => anchor.covers(at, at, col),
            Self::Row(row) => anchor.covers(at, row, usize::from(anchor.col)),
            Self::Layer(z) => anchor.z == z,
        }
    }
}
