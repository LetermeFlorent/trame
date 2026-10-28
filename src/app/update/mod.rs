mod download;
mod installer;
mod launch;
mod replace;
mod route;
mod version;
mod watch;

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use trame::log_error;
use trame::preferences::Updates;

pub use launch::{relaunch, restart, start_again};
use replace::{bundle, staged, swap};
use route::Route;
use version::Version;

const RELEASES: &str = "https://github.com/LetermeFlorent/trame/releases";
const SKIP_VARIABLE: &str = "TRAME_NO_UPDATE";
const UPDATED_FLAG: &str = "--updated";
const INSTALL_FLAG: &str = "--install";

static AUTOMATIC: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    Checking,
    Downloading { version: String, percent: u8 },
    Installing(String),
    Done,
    Installed(PathBuf),
    Available(String),
    Ready { version: String, exe: PathBuf },
}

pub fn allow_automatic(enabled: bool) {
    AUTOMATIC.store(enabled, Ordering::Relaxed);
}

pub fn start(updates: Updates, notify: impl Fn(UpdateStatus) + Send + 'static) -> UpdateStatus {
    let Ok(exe) = env::current_exe() else {
        return UpdateStatus::Done;
    };
    swap::remove_leftover(&bundle::root(&exe).unwrap_or_else(|| exe.clone()));
    allow_automatic(updates.auto);
    let Some(route) = allowed().then(|| Route::of(&exe)).flatten() else {
        return UpdateStatus::Done;
    };
    let installs = route.replaces() && flagged(INSTALL_FLAG) && !flagged(UPDATED_FLAG);
    let spawned = thread::Builder::new()
        .name("trame-update".into())
        .spawn(move || follow(route, installs, exe, &notify));
    match spawned {
        Ok(_) if installs => UpdateStatus::Checking,
        Ok(_) => UpdateStatus::Done,
        Err(error) => {
            log_error!("the update check did not start: {error}");
            UpdateStatus::Done
        }
    }
}

fn allowed() -> bool {
    !cfg!(debug_assertions) && env::var_os(SKIP_VARIABLE).is_none()
}

fn flagged(flag: &str) -> bool {
    env::args_os().any(|arg| arg == flag)
}

fn follow(route: Route, installs: bool, exe: PathBuf, notify: &dyn Fn(UpdateStatus)) {
    staged::tidy(&route.target(&exe));
    if installs {
        let status = check(route, &exe, notify);
        let installed = matches!(status, UpdateStatus::Installed(_));
        notify(status);
        if installed {
            return;
        }
    }
    watch::watch(route, &exe, notify);
}

fn check(route: Route, exe: &Path, notify: &dyn Fn(UpdateStatus)) -> UpdateStatus {
    let staged = staged::newest(&route.target(exe));
    let Some(latest) = staged.or_else(watch::newer_release) else {
        return UpdateStatus::Done;
    };
    match install(route, latest, exe, notify) {
        Ok(()) => UpdateStatus::Installed(exe.to_path_buf()),
        Err(error) => {
            log_error!("the update to {latest} failed: {error}");
            UpdateStatus::Done
        }
    }
}

fn install(
    route: Route,
    version: Version,
    exe: &Path,
    notify: &dyn Fn(UpdateStatus),
) -> io::Result<()> {
    let number = version.number();
    let downloading = |percent| {
        notify(UpdateStatus::Downloading {
            version: number.clone(),
            percent,
        })
    };
    let installing = || notify(UpdateStatus::Installing(number.clone()));
    replace::put_in_place(route, exe, version, Some(&downloading), &installing)
}

fn automatic() -> bool {
    AUTOMATIC.load(Ordering::Relaxed)
}
