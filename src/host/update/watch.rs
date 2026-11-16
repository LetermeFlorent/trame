use std::thread;
use std::time::Duration;

use super::version::Version;
use super::{UpdateStatus, download};

const CHECK_EVERY: Duration = Duration::from_secs(60 * 60);

pub fn newer_release() -> Option<Version> {
    let latest = download::latest()?;
    let newer = Version::current().is_some_and(|current| latest > current);
    newer.then_some(latest)
}

pub fn watch(plain: bool, notify: &dyn Fn(UpdateStatus)) {
    let wait = || {
        thread::sleep(CHECK_EVERY);
        true
    };
    poll_releases(newer_release, wait, plain, notify);
}

pub fn poll_releases(
    mut check: impl FnMut() -> Option<Version>,
    mut wait: impl FnMut() -> bool,
    mut at_start: bool,
    notify: &dyn Fn(UpdateStatus),
) {
    let mut offered = None;
    loop {
        let latest = check();
        match offer(latest, offered) {
            Some(UpdateStatus::Available(number)) if at_start => {
                notify(UpdateStatus::AvailableAtStart(number));
            }
            Some(status) => notify(status),
            None => {}
        }
        at_start = false;
        offered = latest.or(offered);
        if !wait() {
            return;
        }
    }
}

pub fn offer(latest: Option<Version>, offered: Option<Version>) -> Option<UpdateStatus> {
    latest
        .filter(|latest| Some(*latest) != offered)
        .map(|latest| UpdateStatus::Available(latest.number()))
}
