use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Updates {
    #[serde(default = "super::enabled", rename = "update_automatically")]
    pub auto: bool,
}

impl Default for Updates {
    fn default() -> Self {
        Self {
            auto: super::enabled(),
        }
    }
}
