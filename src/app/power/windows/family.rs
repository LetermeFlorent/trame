use std::collections::{HashMap, HashSet};
use std::os::windows::io::AsRawHandle;
use std::time::Instant;

use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::SystemInformation::GetSystemTimeAsFileTime;
use windows_sys::Win32::System::Threading::{
    BELOW_NORMAL_PRIORITY_CLASS, GetPriorityClass, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
};

use super::throttle;
use super::tree::{Member, descendants, time_of};

#[derive(Clone, Copy)]
struct Saved {
    born: u64,
    class: u32,
}

#[derive(Default)]
pub struct Family {
    since: u64,
    saved: HashMap<u32, Saved>,
}

impl Family {
    pub fn apply(&mut self, everyone: bool, shells: &[(u32, bool)]) {
        let resting: HashSet<u32> = shells
            .iter()
            .filter(|(_, resting)| *resting)
            .map(|&(shell, _)| shell)
            .collect();
        if !everyone && resting.is_empty() && self.saved.is_empty() {
            return;
        }
        let restoring = !self.saved.is_empty();
        if !restoring {
            self.since = now();
        }
        let members = descendants();
        let originals = self.originals(&members);
        let mut kept = HashMap::new();
        for member in &members {
            let original = originals[&member.id];
            let handle = member.handle.as_raw_handle();
            let slowed =
                everyone || resting.contains(&member.id) || kept.contains_key(&member.parent);
            if slowed {
                throttle(handle, true, IDLE_PRIORITY_CLASS);
                kept.insert(member.id, original);
            } else if restoring {
                throttle(handle, false, original.class);
            }
        }
        self.saved = kept;
    }

    pub fn retry_at(&self) -> Option<Instant> {
        None
    }

    fn originals(&self, members: &[Member]) -> HashMap<u32, Saved> {
        let mut found = HashMap::new();
        for member in members {
            let kept = self
                .saved
                .get(&member.id)
                .filter(|saved| saved.born == member.born);
            let class = kept.map_or_else(|| self.class_of(member, &found), |saved| saved.class);
            found.insert(
                member.id,
                Saved {
                    born: member.born,
                    class,
                },
            );
        }
        found
    }

    fn class_of(&self, member: &Member, found: &HashMap<u32, Saved>) -> u32 {
        let current = match unsafe { GetPriorityClass(member.handle.as_raw_handle()) } {
            0 => NORMAL_PRIORITY_CLASS,
            class => class,
        };
        if current != IDLE_PRIORITY_CLASS || member.born < self.since {
            return current;
        }
        let parent = found
            .get(&member.parent)
            .map_or(NORMAL_PRIORITY_CLASS, |saved| saved.class);
        match parent {
            IDLE_PRIORITY_CLASS | BELOW_NORMAL_PRIORITY_CLASS => parent,
            _ => NORMAL_PRIORITY_CLASS,
        }
    }
}

fn now() -> u64 {
    let mut time = FILETIME::default();
    unsafe { GetSystemTimeAsFileTime(&mut time) };
    time_of(time)
}
