#[cfg(windows)]
mod windows;

use std::time::{Duration, Instant};

use winit::window::Window;

const RECHECK: Duration = Duration::from_millis(250);

#[derive(Default)]
pub struct Visibility {
    occluded: bool,
    hidden: bool,
    checked: Option<Instant>,
    retry: Option<Instant>,
}

impl Visibility {
    pub fn set_occluded(&mut self, occluded: bool) {
        self.occluded = occluded;
        self.forget();
    }

    pub fn forget(&mut self) {
        self.checked = None;
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
        self.occluded || self.hidden
    }

    pub fn missed(&mut self) {
        self.retry = Some(Instant::now() + RECHECK);
    }

    pub fn drawn(&mut self) {
        self.retry = None;
    }

    pub fn retry(&mut self, window: &Window) -> Option<Instant> {
        let due = self.retry?;
        if Instant::now() < due {
            return Some(due);
        }
        if self.is_hidden(window) {
            self.missed();
            return self.retry;
        }
        self.retry = None;
        window.request_redraw();
        None
    }
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
