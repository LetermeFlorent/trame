use serde::{Deserialize, Serialize};

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
