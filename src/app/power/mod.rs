mod mode;
mod rest;
#[cfg(test)]
mod tests;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::time::{Duration, Instant};

use trame::session::SessionId;

use mode::Mode;
use rest::Rest;
pub use rest::Shown;

#[cfg(unix)]
use unix::{Family, set_own};
#[cfg(windows)]
use windows::{Family, set_own};

const IDLE_AFTER: Duration = Duration::from_secs(30);
const FAMILY_RECHECK: Duration = Duration::from_secs(10);

pub struct Power {
    enabled: bool,
    background: bool,
    last: Instant,
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
        self.background = background;
        self.touch();
    }

    pub fn set_tabs(&mut self, tabs: impl IntoIterator<Item = Shown>) {
        let now = Instant::now();
        self.rest.set_tabs(tabs, now);
        self.apply(now);
    }

    pub fn output(&mut self, id: SessionId) {
        self.rest.output(id, Instant::now());
        self.touch();
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
            Mode::Normal => self.enabled.then(|| self.last + IDLE_AFTER),
            Mode::Own => None,
            Mode::Family => Some(self.walked + FAMILY_RECHECK),
        };
        let others = [self.rest.deadline(), self.family.retry_at()];
        mode.into_iter().chain(others.into_iter().flatten()).min()
    }

    pub fn tick(&mut self, now: Instant) {
        self.apply(now);
    }

    fn apply(&mut self, now: Instant) {
        let idle = now.saturating_duration_since(self.last) >= IDLE_AFTER;
        let next = Mode::wanted(self.enabled, self.background, idle);
        let everyone = next == Mode::Family;
        self.rest.tick(now);
        let toggled = everyone != (self.mode == Mode::Family);
        let stale = everyone && now.saturating_duration_since(self.walked) >= FAMILY_RECHECK;
        let retry = self.family.retry_at().is_some_and(|at| now >= at);
        if self.rest.take_changed() || toggled || stale || retry {
            self.family.apply(everyone, &self.rest.shells());
            self.walked = now;
        }
        if (next == Mode::Normal) != (self.mode == Mode::Normal) {
            set_own(next != Mode::Normal);
        }
        self.mode = next;
    }
}

impl Drop for Power {
    fn drop(&mut self) {
        self.enabled = false;
        self.rest.set_enabled(false, Instant::now());
        self.apply(Instant::now());
    }
}
