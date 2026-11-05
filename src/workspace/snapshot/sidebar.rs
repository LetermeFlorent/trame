use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ui::display::SidebarMode;

#[derive(Serialize, Deserialize)]
struct StoredSidebar {
    #[serde(default)]
    sidebar: Option<SidebarMode>,
    #[serde(default)]
    sidebar_open: Option<bool>,
}

pub(super) fn serialize<S: Serializer>(
    mode: &SidebarMode,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    StoredSidebar {
        sidebar: Some(*mode),
        sidebar_open: Some(*mode == SidebarMode::Open),
    }
    .serialize(serializer)
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<SidebarMode, D::Error> {
    let stored = StoredSidebar::deserialize(deserializer)?;
    match (stored.sidebar, stored.sidebar_open) {
        (Some(mode), _) => Ok(mode),
        (None, Some(true)) => Ok(SidebarMode::Open),
        (None, Some(false)) => Ok(SidebarMode::Rail),
        (None, None) => Err(D::Error::missing_field("sidebar")),
    }
}
