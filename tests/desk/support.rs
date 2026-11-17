use std::sync::Arc;

use trame::desk::{Desk, DeskSetup};
use trame::host::persist::Slot;
use trame::preferences::Preferences;
use trame::session::Notifier;
use trame::theme::Appearance;

use crate::sandbox::Sandbox;

pub fn desk() -> Desk {
    desk_with(Arc::new(|_| {}))
}

pub fn desk_with(notifier: Notifier) -> Desk {
    desk_at(Slot::new(None, 0), notifier)
}

pub fn desk_in(sandbox: &Sandbox) -> Desk {
    desk_at(Slot::new(Some(sandbox.root.as_path()), 0), Arc::new(|_| {}))
}

fn desk_at(slot: Slot, notifier: Notifier) -> Desk {
    Desk::new(DeskSetup {
        preferences: Preferences::default(),
        slot,
        notifier,
        appearance: Appearance::Dark,
        icon_dir: None,
    })
}
