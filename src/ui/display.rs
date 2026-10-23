use serde::{Deserialize, Serialize};

const SPACIOUS: f32 = 1.25;

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

    pub fn spacing(self) -> f32 {
        match self {
            Self::Compact => 1.0,
            Self::Spacious => SPACIOUS,
        }
    }
}
