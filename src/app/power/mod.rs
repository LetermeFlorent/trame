mod mode;
#[cfg(not(windows))]
mod other;
#[cfg(windows)]
mod windows;

use std::time::{Duration, Instant};

use mode::Mode;

#[cfg(not(windows))]
use other::{Family, set_own};
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
    family: Family,
}

impl Power {
    pub fn new(enabled: bool) -> Self {
        let now = Instant::now();
        Self {
            enabled,
            background: false,
            last: now,
            walked: now,
            mode: Mode::Normal,
            family: Family::default(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.touch();
    }

    pub fn set_background(&mut self, background: bool) {
        self.background = background;
        self.touch();
    }

    pub fn touch(&mut self) {
        self.last = Instant::now();
        self.apply(self.last);
    }

    pub fn deadline(&self) -> Option<Instant> {
        match self.mode {
            Mode::Normal => self.enabled.then(|| self.last + IDLE_AFTER),
            Mode::Own => None,
            Mode::Family => Some(self.walked + FAMILY_RECHECK),
        }
    }

    pub fn tick(&mut self, now: Instant) {
        self.apply(now);
    }

    fn apply(&mut self, now: Instant) {
        let idle = now.saturating_duration_since(self.last) >= IDLE_AFTER;
        let next = Mode::wanted(self.enabled, self.background, idle);
        let stale = now.saturating_duration_since(self.walked) >= FAMILY_RECHECK;
        if next == Mode::Family && (self.mode != Mode::Family || stale) {
            self.family.throttle();
            self.walked = now;
        } else if next != Mode::Family && self.mode == Mode::Family {
            self.family.restore();
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
        self.apply(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_background_slows_the_whole_family_even_while_output_arrives() {
        let mut power = Power::new(true);
        power.set_background(true);
        power.touch();
        assert_eq!(power.mode, Mode::Family);
        assert_eq!(power.deadline(), Some(power.walked + FAMILY_RECHECK));
    }

    #[test]
    fn coming_back_to_the_front_waits_for_idleness_again() {
        let mut power = Power::new(true);
        power.set_background(true);
        power.set_background(false);
        assert_eq!(power.mode, Mode::Normal);
        assert_eq!(power.deadline(), Some(power.last + IDLE_AFTER));
    }

    #[test]
    fn a_quiet_window_in_front_only_slows_trame() {
        let mut power = Power::new(true);
        power.tick(power.last + IDLE_AFTER);
        assert_eq!(power.mode, Mode::Own);
        assert_eq!(power.deadline(), None);
        power.touch();
        assert_eq!(power.mode, Mode::Normal);
    }

    #[test]
    fn the_switch_turned_off_keeps_full_speed_everywhere() {
        let mut power = Power::new(true);
        power.set_background(true);
        power.set_enabled(false);
        assert_eq!(power.mode, Mode::Normal);
        assert_eq!(power.deadline(), None);
        power.tick(power.last + IDLE_AFTER);
        assert_eq!(power.mode, Mode::Normal);
    }
}
