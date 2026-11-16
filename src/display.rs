use serde::{Deserialize, Serialize};

pub const TERMINAL_FONT_SIZE: f32 = 14.0;
pub const MIN_TERMINAL_FONT_SIZE: f32 = 8.0;
pub const MAX_TERMINAL_FONT_SIZE: f32 = 32.0;
pub const SPACING: f32 = 1.25;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelSide {
    #[default]
    Left,
    Right,
}

impl PanelSide {
    pub const ALL: [Self; 2] = [Self::Left, Self::Right];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Density {
    #[default]
    Compact,
    Spacious,
}

impl Density {
    pub const ALL: [Self; 2] = [Self::Compact, Self::Spacious];

    pub fn cards(self) -> bool {
        self == Self::Spacious
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarMode {
    Hidden,
    Rail,
    #[default]
    Open,
}

impl SidebarMode {
    pub fn next(self) -> Self {
        match self {
            Self::Open => Self::Rail,
            Self::Rail => Self::Hidden,
            Self::Hidden => Self::Open,
        }
    }

    pub fn extent(self) -> f32 {
        f32::from(self as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Mac,
    Linux,
}

impl Platform {
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Mac
        } else {
            Self::Linux
        }
    }

    pub fn draws_window_controls(self) -> bool {
        self != Self::Mac
    }
}
