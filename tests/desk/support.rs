use std::sync::Arc;

use trame::desk::{Desk, DeskSetup};
use trame::host::persist::Slot;
use trame::preferences::Preferences;
use trame::theme::Appearance;

pub fn desk() -> Desk {
    Desk::new(DeskSetup {
        preferences: Preferences::default(),
        slot: Slot::new(None, 0),
        notifier: Arc::new(|_| {}),
        appearance: Appearance::Dark,
        icon_dir: None,
    })
}
