use std::path::Path;
use std::thread;
use std::time::Duration;

use super::route::Route;
use super::version::Version;
use super::{UpdateStatus, ahead, fetch, setup, staged};

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
            if offered != Some(latest) {
                notify(UpdateStatus::Available(latest.number()));
                offered = Some(latest);
            }
            let fetched = ahead().then(|| staged::fetch(exe, latest, asset));
            if let Some(Err(error)) = fetched {
                eprintln!("trame: the early download of {latest} failed: {error}");
            }
        }
        thread::sleep(CHECK_EVERY);
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
