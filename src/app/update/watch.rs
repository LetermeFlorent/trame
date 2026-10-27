use std::path::Path;
use std::thread;
use std::time::Duration;

use super::route::Route;
use super::version::Version;
use super::{UpdateStatus, automatic, fetch, setup, staged, swap};

const CHECK_EVERY: Duration = Duration::from_secs(60 * 60);

pub fn newer_release() -> Option<Version> {
    let latest = fetch::latest()?;
    let newer = Version::current().is_some_and(|current| latest > current);
    newer.then_some(latest)
}

pub fn watch(route: Route, exe: &Path, asset: &str, notify: &dyn Fn(UpdateStatus)) {
    if route == Route::Setup {
        return watch_installer(notify);
    }
    let mut offered = None;
    loop {
        if let Some(latest) = newer_release() {
            let status = match automatic() {
                true => put_in_place(exe, latest, asset),
                false => UpdateStatus::Available(latest.number()),
            };
            if offered.as_ref() != Some(&status) {
                notify(status.clone());
            }
            if matches!(status, UpdateStatus::Ready { .. }) {
                return;
            }
            offered = Some(status);
        }
        thread::sleep(CHECK_EVERY);
    }
}

fn put_in_place(exe: &Path, version: Version, asset: &str) -> UpdateStatus {
    let staged = staged::path(exe, version);
    let placed = staged::fetch(exe, version, asset).and_then(|()| swap::replace(exe, &staged));
    match placed {
        Ok(()) => UpdateStatus::Ready {
            version: version.number(),
            exe: exe.to_path_buf(),
        },
        Err(error) => {
            eprintln!("trame: the update to {version} failed: {error}");
            UpdateStatus::Available(version.number())
        }
    }
}

fn watch_installer(notify: &dyn Fn(UpdateStatus)) {
    if installer_offered(notify) {
        return;
    }
    setup::clear();
    loop {
        thread::sleep(CHECK_EVERY);
        if installer_offered(notify) {
            return;
        }
    }
}

fn installer_offered(notify: &dyn Fn(UpdateStatus)) -> bool {
    let Some(latest) = newer_release() else {
        return false;
    };
    match setup::fetch(latest) {
        Ok(()) => {
            notify(UpdateStatus::Available(latest.number()));
            true
        }
        Err(error) => {
            eprintln!("trame: the installer of {latest} failed: {error}");
            false
        }
    }
}
