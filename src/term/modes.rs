#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modes(u32);

impl Modes {
    pub const APP_CURSOR: Self = Self(1);
    pub const ORIGIN: Self = Self(1 << 2);
    pub const AUTOWRAP: Self = Self(1 << 3);
    pub const CURSOR_VISIBLE: Self = Self(1 << 4);
    pub const INSERT: Self = Self(1 << 5);
    pub const NEW_LINE: Self = Self(1 << 6);
    pub const ALT_SCREEN: Self = Self(1 << 7);
    pub const BRACKETED_PASTE: Self = Self(1 << 8);
    pub const FOCUS_EVENTS: Self = Self(1 << 9);
    pub const MOUSE_CLICK: Self = Self(1 << 10);
    pub const MOUSE_DRAG: Self = Self(1 << 11);
    pub const MOUSE_MOTION: Self = Self(1 << 12);
    pub const MOUSE_SGR: Self = Self(1 << 13);
    pub const ALTERNATE_SCROLL: Self = Self(1 << 14);
    pub const SYNC_UPDATE: Self = Self(1 << 15);
    pub const WIN32_INPUT: Self = Self(1 << 16);
    pub const MOUSE_ANY: Self =
        Self(Self::MOUSE_CLICK.0 | Self::MOUSE_DRAG.0 | Self::MOUSE_MOTION.0);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn set(&mut self, other: Self, enabled: bool) {
        if enabled {
            self.0 |= other.0;
        } else {
            self.0 &= !other.0;
        }
    }
}

impl Default for Modes {
    fn default() -> Self {
        Self(Self::AUTOWRAP.0 | Self::CURSOR_VISIBLE.0 | Self::ALTERNATE_SCROLL.0)
    }
}
