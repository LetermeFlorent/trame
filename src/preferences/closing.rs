use super::Preferences;

pub const CONFIRM_FROM_TABS: usize = 2;

impl Preferences {
    pub fn toggle_confirm_close(&self) -> Self {
        self.with(|next| next.confirm_close = !next.confirm_close)
    }

    pub fn confirms_closing(&self, tabs: usize) -> bool {
        self.confirm_close && tabs >= CONFIRM_FROM_TABS
    }
}
