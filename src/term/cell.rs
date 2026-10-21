#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Foreground,
    Background,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags(u16);

impl Flags {
    pub const BOLD: Self = Self(1);
    pub const DIM: Self = Self(1 << 1);
    pub const ITALIC: Self = Self(1 << 2);
    pub const UNDERLINE: Self = Self(1 << 3);
    pub const DOUBLE_UNDERLINE: Self = Self(1 << 4);
    pub const CURLY_UNDERLINE: Self = Self(1 << 5);
    pub const INVERSE: Self = Self(1 << 6);
    pub const HIDDEN: Self = Self(1 << 7);
    pub const STRIKE: Self = Self(1 << 8);
    pub const WIDE: Self = Self(1 << 9);
    pub const WIDE_SPACER: Self = Self(1 << 10);
    pub const ANY_UNDERLINE: Self =
        Self(Self::UNDERLINE.0 | Self::DOUBLE_UNDERLINE.0 | Self::CURLY_UNDERLINE.0);
    pub const WIDTH_MARKS: Self = Self(Self::WIDE.0 | Self::WIDE_SPACER.0);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    pub fn set(&mut self, other: Self, enabled: bool) {
        if enabled {
            self.insert(other);
        } else {
            self.remove(other);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub flags: Flags,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Foreground,
            bg: Color::Background,
            flags: Flags::empty(),
        }
    }
}

impl Cell {
    pub fn blank_with(template: &Cell) -> Self {
        Self {
            bg: template.bg,
            ..Self::default()
        }
    }

    pub fn is_blank(&self) -> bool {
        (self.ch == ' ' || self.ch == '\0')
            && self.bg == Color::Background
            && !self.flags.intersects(Flags::INVERSE)
            && !self.flags.intersects(Flags::ANY_UNDERLINE)
    }
}
