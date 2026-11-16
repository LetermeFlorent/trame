use std::time::Instant;

use super::{Mode, Power};

impl Power {
    #[doc(hidden)]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    #[doc(hidden)]
    pub fn last(&self) -> Instant {
        self.last
    }

    #[doc(hidden)]
    pub fn walked(&self) -> Instant {
        self.walked
    }

    #[doc(hidden)]
    pub fn shells(&self) -> Vec<(u32, bool)> {
        self.rest.shells()
    }
}
