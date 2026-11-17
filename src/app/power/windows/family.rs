use std::collections::{HashMap, HashSet};
use std::os::windows::io::{AsRawHandle, OwnedHandle};
use std::time::Instant;

use windows_sys::Win32::System::Threading::{
    BELOW_NORMAL_PRIORITY_CLASS, GetPriorityClass, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    PROCESS_POWER_THROTTLING_STATE,
};

use super::tree::{Member, descendants};
use super::{LEFT_TO_WINDOWS, eco, speed_of, throttle};

struct Saved {
    handle: OwnedHandle,
    since: u64,
    class: u32,
    speed: PROCESS_POWER_THROTTLING_STATE,
}

#[derive(Default)]
pub struct Family {
    saved: HashMap<u32, Saved>,
}

impl Family {
    pub fn apply(&mut self, everyone: bool, shells: &[(u32, bool)]) {
        let wanted: HashSet<u32> = shells
            .iter()
            .filter(|&&(_, resting)| everyone || resting)
            .map(|&(shell, _)| shell)
            .collect();
        if wanted.is_empty() && self.saved.is_empty() {
            return;
        }
        let roots = self.saved.iter().map(|(&id, saved)| (id, saved.since));
        let (taken, members) = descendants(roots);
        let mut left = std::mem::take(&mut self.saved);
        let mut restored = HashMap::new();
        for member in members {
            let id = member.id;
            let slow = wanted.contains(&id) || self.saved.contains_key(&member.parent);
            let kept = left.remove(&id);
            let parent = [&self.saved, &restored, &left]
                .into_iter()
                .find_map(|saved| saved.get(&member.parent));
            let Some(saved) = kept.or_else(|| Saved::first(member, slow, parent, taken)) else {
                continue;
            };
            if slow {
                throttle(saved.handle.as_raw_handle(), eco(true), IDLE_PRIORITY_CLASS);
                self.saved.insert(id, saved);
            } else {
                saved.restore();
                restored.insert(id, saved);
            }
        }
        left.values().for_each(Saved::restore);
    }

    pub fn retry_at(&self) -> Option<Instant> {
        None
    }
}

impl Saved {
    fn first(member: Member, slow: bool, parent: Option<&Saved>, taken: u64) -> Option<Self> {
        let heir = parent.filter(|parent| member.born >= parent.since);
        if !slow && heir.is_none() {
            return None;
        }
        let handle = member.handle.as_raw_handle();
        let current = match unsafe { GetPriorityClass(handle) } {
            0 => NORMAL_PRIORITY_CLASS,
            class => class,
        };
        let inherited = heir.filter(|_| current == IDLE_PRIORITY_CLASS);
        if !slow && inherited.is_none() {
            return None;
        }
        let (since, class) = match inherited {
            Some(parent) => (member.born, passed_down(parent.class)),
            None => (taken, current),
        };
        Some(Self {
            since,
            class,
            speed: speed_of(handle).unwrap_or(LEFT_TO_WINDOWS),
            handle: member.handle,
        })
    }

    fn restore(&self) {
        throttle(self.handle.as_raw_handle(), self.speed, self.class);
    }
}

fn passed_down(class: u32) -> u32 {
    match class {
        IDLE_PRIORITY_CLASS | BELOW_NORMAL_PRIORITY_CLASS => class,
        _ => NORMAL_PRIORITY_CLASS,
    }
}
