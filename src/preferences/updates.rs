use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Updates {
    #[serde(default, rename = "update_at_launch")]
    pub auto: bool,
    #[serde(default = "ahead", rename = "update_ahead")]
    pub ahead: bool,
}

impl Default for Updates {
    fn default() -> Self {
        Self {
            auto: false,
            ahead: ahead(),
        }
    }
}

impl Updates {
    pub fn downloads_ahead(self) -> bool {
        self.auto && self.ahead
    }
}

fn ahead() -> bool {
    true
}
