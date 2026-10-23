use std::thread;
use std::time::Duration;

use super::UpdateStatus;
use super::fetch;
use super::version::Version;

const CHECK_EVERY: Duration = Duration::from_secs(60 * 60);

pub fn newer_release() -> Option<Version> {
    let latest = fetch::latest()?;
    let newer = Version::current().is_some_and(|current| latest > current);
    newer.then_some(latest)
}

pub fn watch(notify: &dyn Fn(UpdateStatus)) {
    loop {
        thread::sleep(CHECK_EVERY);
        if let Some(latest) = newer_release() {
            notify(UpdateStatus::Available(latest.number()));
            return;
        }
    }
}
