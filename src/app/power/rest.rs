use std::collections::HashMap;
use std::time::{Duration, Instant};

use trame::session::SessionId;

pub const REST_AFTER: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shown {
    pub id: SessionId,
    pub shell: u32,
    pub shown: bool,
}

struct Tab {
    shell: u32,
    shown: bool,
    quiet_since: Instant,
    resting: bool,
}

#[derive(Default)]
pub struct Rest {
    enabled: bool,
    tabs: HashMap<SessionId, Tab>,
    changed: bool,
    rested: Vec<SessionId>,
}

impl Rest {
    pub fn set_enabled(&mut self, enabled: bool, now: Instant) {
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        for tab in self.tabs.values_mut() {
            tab.quiet_since = now;
        }
    }

    pub fn set_tabs(&mut self, tabs: impl IntoIterator<Item = Shown>, now: Instant) {
        let mut next = HashMap::new();
        for Shown { id, shell, shown } in tabs {
            let tab = match self.tabs.remove(&id) {
                Some(tab) => Tab {
                    shell,
                    shown,
                    quiet_since: match tab.shown && !shown {
                        true => now,
                        false => tab.quiet_since,
                    },
                    resting: tab.resting,
                },
                None => Tab {
                    shell,
                    shown,
                    quiet_since: now,
                    resting: false,
                },
            };
            next.insert(id, tab);
        }
        self.changed |= self.tabs.values().any(|gone| gone.resting);
        self.tabs = next;
    }

    pub fn output(&mut self, id: SessionId, now: Instant) {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.quiet_since = now;
        }
    }

    pub fn tick(&mut self, now: Instant) {
        for (id, tab) in &mut self.tabs {
            let resting = self.enabled && !tab.shown && now >= tab.quiet_since + REST_AFTER;
            if resting == tab.resting {
                continue;
            }
            tab.resting = resting;
            self.changed = true;
            if resting {
                self.rested.push(*id);
            }
        }
    }

    pub fn deadline(&self) -> Option<Instant> {
        let waiting = self.tabs.values().filter(|tab| !tab.shown && !tab.resting);
        let due = waiting.map(|tab| tab.quiet_since + REST_AFTER).min();
        due.filter(|_| self.enabled)
    }

    pub fn shells(&self) -> Vec<(u32, bool)> {
        self.tabs
            .values()
            .map(|tab| (tab.shell, tab.resting))
            .collect()
    }

    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    pub fn take_rested(&mut self) -> Vec<SessionId> {
        std::mem::take(&mut self.rested)
    }
}
