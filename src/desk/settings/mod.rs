mod apply;

use std::mem;

use crate::preferences::Preferences;

use super::{Desk, Effect, Effects};

impl Desk {
    pub fn store_preferences(&mut self, preferences: Preferences) -> Effects {
        let preferences = preferences.sanitized();
        if preferences == self.preferences {
            return Effects::default();
        }
        let previous = mem::replace(&mut self.preferences, preferences);
        self.apply_preferences(&previous);
        self.saver.touch();
        let current = &self.preferences;
        self.strings = current.language.resolve().strings();
        let layout = |kept: &Preferences| {
            let sizes = (kept.font_size, kept.interface_size);
            (sizes, kept.panel_side, kept.density)
        };
        match layout(&previous) == layout(current) {
            true => Effect::Changed.into(),
            false => Effect::Rescale.into(),
        }
    }

    pub fn follow_preferences(&mut self, preferences: &Preferences) -> Effects {
        if preferences == &self.preferences {
            return Effects::default();
        }
        self.store_preferences(preferences.clone())
    }
}
