use std::collections::HashMap;
use std::os::windows::io::AsRawHandle;

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
    pub fn throttle(&mut self) {
        if self.saved.is_empty() {
            self.since = now();
        }
        let members = descendants();
        let originals = self.originals(&members);
        self.saved.extend(originals);
        for member in &members {
            throttle(member.handle.as_raw_handle(), true, IDLE_PRIORITY_CLASS);
        }
    }

    pub fn restore(&mut self) {
        let members = descendants();
        let originals = self.originals(&members);
        for member in &members {
            let class = originals[&member.id].class;
            throttle(member.handle.as_raw_handle(), false, class);
        }
        self.saved.clear();
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
