use std::path::Path;
use std::thread;
use std::time::Duration;

use trame::log_error;

use super::route::Route;
use super::version::Version;
use super::{UpdateStatus, automatic, download, installer, replace};

const CHECK_EVERY: Duration = Duration::from_secs(60 * 60);

pub fn newer_release() -> Option<Version> {
    let latest = download::latest()?;
    let newer = Version::current().is_some_and(|current| latest > current);
    newer.then_some(latest)
}

pub fn watch(route: Route, exe: &Path, notify: &dyn Fn(UpdateStatus)) {
    if !route.replaces() {
        return watch_installer(route, notify);
    }
    let mut offered = None;
    loop {
        if let Some(latest) = newer_release() {
            let status = match automatic() {
                true => put_in_place(route, exe, latest),
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

fn put_in_place(route: Route, exe: &Path, version: Version) -> UpdateStatus {
    match replace::put_in_place(route, exe, version, None, &|| {}) {
        Ok(()) => UpdateStatus::Ready {
            version: version.number(),
            exe: exe.to_path_buf(),
        },
        Err(error) => {
            log_error!("the update to {version} failed: {error}");
            UpdateStatus::Available(version.number())
        }
    }
}

fn watch_installer(route: Route, notify: &dyn Fn(UpdateStatus)) {
    if installer_offered(route, notify) {
        return;
    }
    installer::clear(route);
    loop {
        thread::sleep(CHECK_EVERY);
        if installer_offered(route, notify) {
            return;
        }
    }
}

fn installer_offered(route: Route, notify: &dyn Fn(UpdateStatus)) -> bool {
    let Some(latest) = newer_release() else {
        return false;
    };
    match installer::fetch(route, latest) {
        Ok(()) => {
            notify(UpdateStatus::Available(latest.number()));
            true
        }
        Err(error) => {
            log_error!("the installer of {latest} failed: {error}");
            false
        }
    }
}
