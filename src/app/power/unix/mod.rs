#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

use std::collections::HashSet;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use linux as os;
#[cfg(target_os = "macos")]
use macos as os;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod os {
    pub fn slow(_shell: u32, _on: bool) -> bool {
        true
    }

    pub fn set_own(_on: bool) {}
}

pub use os::set_own;

const RETRY_AFTER: Duration = Duration::from_millis(150);
const FIRST_USER_PID: u32 = 2;

#[derive(Default)]
pub struct Family {
    slowed: HashSet<u32>,
    retry: Option<Instant>,
}

impl Family {
    pub fn apply(&mut self, everyone: bool, shells: &[(u32, bool)]) {
        self.retry = None;
        let wanted: HashSet<u32> = shells
            .iter()
            .filter(|&&(shell, resting)| shell >= FIRST_USER_PID && (everyone || resting))
            .map(|&(shell, _)| shell)
            .collect();
        let slowing = wanted.difference(&self.slowed).map(|&shell| (shell, true));
        let waking = self.slowed.difference(&wanted).map(|&shell| (shell, false));
        let changes: Vec<(u32, bool)> = slowing.chain(waking).collect();
        for (shell, slow) in changes {
            if !os::slow(shell, slow) {
                self.retry = Some(Instant::now() + RETRY_AFTER);
                return;
            }
            match slow {
                true => self.slowed.insert(shell),
                false => self.slowed.remove(&shell),
            };
        }
    }

    pub fn retry_at(&self) -> Option<Instant> {
        self.retry
    }
}
