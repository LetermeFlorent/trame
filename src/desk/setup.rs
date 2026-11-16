use std::path::PathBuf;

use crate::files::IconStore;
use crate::host::persist::{Saver, Slot};
use crate::host::power::Power;
use crate::preferences::Preferences;
use crate::session::{Notifier, Transcripts};
use crate::theme::{Appearance, Theme};
use crate::workspace::Workspace;

use super::{Desk, Restarts, Sessions, UpdateState};

pub struct DeskSetup {
    pub preferences: Preferences,
    pub slot: Slot,
    pub notifier: Notifier,
    pub appearance: Appearance,
    pub icon_dir: Option<PathBuf>,
}

impl Desk {
    pub fn new(setup: DeskSetup) -> Self {
        let DeskSetup {
            preferences,
            slot,
            notifier,
            appearance,
            icon_dir,
        } = setup;
        let mut sessions = Sessions::new(notifier);
        sessions.follow_preferences(&preferences);
        let saver = Saver::new(slot);
        Self {
            workspace: Workspace::numbered(preferences.numbering),
            sessions,
            transcripts: Transcripts::new(saver.transcript_dir()),
            saver,
            icons: IconStore::new(icon_dir),
            power: Power::new(preferences.power_switches()),
            theme: Theme::new(appearance),
            strings: preferences.language.resolve().strings(),
            preferences,
            update: UpdateState::default(),
            restarts: Restarts::default(),
            quit: None,
            starting: Vec::new(),
        }
    }
}
