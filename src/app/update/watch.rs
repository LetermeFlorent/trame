use std::thread;
use std::time::Duration;

use super::UpdateStatus;
use super::fetch;
use super::route::Route;
use super::setup;
use super::version::Version;

const CHECK_EVERY: Duration = Duration::from_secs(60 * 60);

pub fn newer_release() -> Option<Version> {
    let latest = fetch::latest()?;
    let newer = Version::current().is_some_and(|current| latest > current);
    newer.then_some(latest)
}

pub fn watch(route: Route, notify: &dyn Fn(UpdateStatus)) {
    if route == Route::Setup {
        if offered(route, notify) {
            return;
        }
        setup::clear();
    }
    loop {
        thread::sleep(CHECK_EVERY);
        if offered(route, notify) {
            return;
        }
    }
}

fn offered(route: Route, notify: &dyn Fn(UpdateStatus)) -> bool {
    let Some(latest) = newer_release() else {
        return false;
    };
    let ready = match route {
        Route::Swap => true,
        Route::Setup => setup::fetch(latest)
            .inspect_err(|error| eprintln!("trame: the installer of {latest} failed: {error}"))
            .is_ok(),
    };
    if ready {
        notify(UpdateStatus::Available(latest.number()));
    }
    ready
}
