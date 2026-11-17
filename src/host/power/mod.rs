mod mode;
mod probe;
mod process;
mod rest;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::time::{Duration, Instant};

use crate::session::SessionId;

pub use mode::Mode;
pub use process::Process;
use rest::Rest;
pub use rest::{RECHECKS_RESTING_SHELLS, REST_AFTER, Shown};

#[cfg(unix)]
use unix::{Family, set_own};
#[cfg(windows)]
use windows::{Family, set_own};

pub const IDLE_AFTER: Duration = Duration::from_secs(30);
pub const FAMILY_RECHECK: Duration = Duration::from_secs(10);

pub struct Power {
    enabled: bool,
    background: bool,
    last: Instant,
    heard: Instant,
    walked: Instant,
    mode: Mode,
    rest: Rest,
    family: Family,
}

impl Power {
    pub fn new([enabled, resting]: [bool; 2]) -> Self {
        let now = Instant::now();
        let mut rest = Rest::default();
        rest.set_enabled(enabled && resting, now);
        Self {
            enabled,
            background: false,
            last: now,
            heard: now,
            walked: now,
            mode: Mode::Normal,
            rest,
            family: Family::default(),
        }
    }

    pub fn set_enabled(&mut self, [enabled, resting]: [bool; 2]) {
        self.enabled = enabled;
        self.rest.set_enabled(enabled && resting, Instant::now());
        self.touch();
    }

    pub fn set_background(&mut self, background: bool) {
        if self.background != background {
            self.background = background;
            self.apply(Instant::now());
        }
    }

    pub fn set_tabs(&mut self, tabs: impl IntoIterator<Item = Shown>) {
        let now = Instant::now();
        self.rest.set_tabs(tabs, now);
        self.apply(now);
    }

    pub fn output(&mut self, id: SessionId, program_resting: bool) {
        if !program_resting {
            self.heard = Instant::now();
            self.rest.output(id, self.heard);
            self.touch();
        }
    }

    pub fn touch(&mut self) {
        self.last = Instant::now();
        self.apply(self.last);
    }

    pub fn take_rested(&mut self) -> Vec<SessionId> {
        self.rest.take_rested()
    }

    pub fn deadline(&self) -> Option<Instant> {
        let mode = match self.mode {
            Mode::Normal if self.background => self.enabled.then(|| self.heard + REST_AFTER),
            Mode::Normal => self.enabled.then(|| self.last + IDLE_AFTER),
            Mode::Own => None,
            Mode::Family => Some(self.walked + FAMILY_RECHECK),
        };
        let others = [self.rest.deadline(self.walked), self.family.retry_at()];
        mode.into_iter().chain(others.into_iter().flatten()).min()
    }

    pub fn tick(&mut self, now: Instant) {
        self.apply(now);
    }

    fn apply(&mut self, now: Instant) {
        let idle = now.saturating_duration_since(self.last) >= IDLE_AFTER;
        let quiet = now.saturating_duration_since(self.heard) >= REST_AFTER;
        let next = Mode::wanted(self.enabled, self.background && quiet, idle);
        let everyone = next == Mode::Family;
        self.rest.tick(now);
        let toggled = everyone != (self.mode == Mode::Family);
        let stale = (everyone || self.rest.rechecks()) && now >= self.walked + FAMILY_RECHECK;
        let retry = self.family.retry_at().is_some_and(|at| now >= at);
        if self.rest.take_changed() || toggled || stale || retry {
            self.family.apply(everyone, &self.rest.shells());
            self.walked = now;
        }
        self.mode = next;
    }

    pub fn rests(&self) -> bool {
        self.mode != Mode::Normal
    }

    pub fn hidden(&self) -> bool {
        self.enabled && self.background
    }
}

impl Drop for Power {
    fn drop(&mut self) {
        self.enabled = false;
        self.rest.set_enabled(false, Instant::now());
        self.apply(Instant::now());
    }
}
