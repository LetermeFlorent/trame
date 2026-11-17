use std::sync::Arc;

use trame::desk::{Desk, DeskSetup};
use trame::host::persist::Slot;
use trame::preferences::Preferences;
use trame::session::Notifier;
use trame::theme::Appearance;

pub fn desk() -> Desk {
    desk_with(Arc::new(|_| {}))
}

pub fn desk_with(notifier: Notifier) -> Desk {
    Desk::new(DeskSetup {
        preferences: Preferences::default(),
        slot: Slot::new(None, 0),
        notifier,
        appearance: Appearance::Dark,
        icon_dir: None,
    })
}
