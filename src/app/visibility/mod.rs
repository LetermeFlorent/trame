#[cfg(test)]
mod tests;
#[cfg(windows)]
mod windows;

use std::time::{Duration, Instant};

use winit::window::Window;

const RECHECK: Duration = Duration::from_millis(250);
const LONGEST_RETRY: Duration = Duration::from_secs(2);

#[derive(Default)]
pub struct Visibility {
    occluded: bool,
    hidden: bool,
    checked: Option<Instant>,
    retry: Option<Instant>,
    misses: u32,
}

impl Visibility {
    pub fn set_occluded(&mut self, occluded: bool) {
        self.occluded = occluded;
        self.forget();
    }

    pub fn forget(&mut self) {
        self.checked = None;
        self.misses = 0;
        if self.retry.is_some() {
            self.retry = Some(Instant::now());
        }
    }

    pub fn is_hidden(&mut self, window: &Window) -> bool {
        let now = Instant::now();
        if self
            .checked
            .is_none_or(|at| now.duration_since(at) >= RECHECK)
        {
            self.hidden = probe(window);
            self.checked = Some(now);
        }
        self.seen_hidden()
    }

    pub fn seen_hidden(&self) -> bool {
        self.occluded || self.hidden
    }

    pub fn missed(&mut self) {
        self.missed_at(Instant::now());
    }

    pub fn drawn(&mut self) {
        self.retry = None;
        self.misses = 0;
    }

    pub fn retry(&mut self, window: &Window) -> Option<Instant> {
        let now = Instant::now();
        if self.retry.is_none_or(|due| now < due) {
            return self.retry;
        }
        if self.is_hidden(window) {
            return Some(self.still_hidden(now));
        }
        self.retry = None;
        window.request_redraw();
        None
    }

    fn missed_at(&mut self, now: Instant) {
        if self.retry.is_none() {
            self.retry = Some(now + retry_wait(self.misses));
        }
    }

    fn still_hidden(&mut self, now: Instant) -> Instant {
        self.misses = self.misses.saturating_add(1);
        let due = now + retry_wait(self.misses);
        self.retry = Some(due);
        due
    }
}

fn retry_wait(misses: u32) -> Duration {
    RECHECK
        .saturating_mul(2u32.saturating_pow(misses))
        .min(LONGEST_RETRY)
}

fn probe(window: &Window) -> bool {
    window.is_minimized() == Some(true) || platform_hidden(window)
}

#[cfg(windows)]
use windows::is_hidden as platform_hidden;

#[cfg(not(windows))]
fn platform_hidden(_window: &Window) -> bool {
    false
}
