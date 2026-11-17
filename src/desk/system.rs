use std::time::Instant;

use crate::host::power::Shown;
use crate::theme::{Appearance, Theme};

use super::{Desk, Effect, Effects, Grids};

impl Desk {
    pub fn tick(&mut self, now: Instant) {
        self.power.tick(now);
        self.compact_rested();
    }

    pub fn sync_rest(&mut self, grids: &Grids) {
        let shown = grids.shown();
        let tabs = self.sessions.map().iter().filter_map(|(&id, session)| {
            let shell = session.shell_id()?;
            Some(Shown {
                id,
                shell,
                shown: shown.contains(&id),
            })
        });
        self.power.set_tabs(tabs);
    }

    pub fn set_appearance(&mut self, appearance: Appearance) -> Effects {
        if appearance == self.theme.appearance {
            return Effects::default();
        }
        self.theme = Theme::new(appearance);
        self.sessions.set_colors(self.theme.reported_colors());
        Effect::Redraw.into()
    }

    pub fn rests(&self) -> bool {
        self.power.rests()
    }

    pub fn hidden(&self) -> bool {
        self.power.hidden()
    }

    fn compact_rested(&mut self) {
        for id in self.power.take_rested() {
            if let Some(session) = self.sessions.get(id) {
                session.terminal().compact();
            }
        }
    }
}
