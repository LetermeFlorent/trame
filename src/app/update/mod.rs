mod fetch;
mod launch;
mod progress;
mod route;
mod setup;
mod staged;
mod swap;
mod version;
mod watch;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use trame::preferences::Updates;

pub use launch::{relaunch, restart, start_again};
use route::Route;
use version::Version;

const RELEASES: &str = "https://github.com/LetermeFlorent/trame/releases";
const SKIP_VARIABLE: &str = "TRAME_NO_UPDATE";
const UPDATED_FLAG: &str = "--updated";
const INSTALL_FLAG: &str = "--install";
const NEW_SUFFIX: &str = ".new";

#[cfg(windows)]
const ASSET: Option<&str> = Some("trame.exe");
#[cfg(not(windows))]
const ASSET: Option<&str> = None;

static AHEAD: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    Checking,
    Downloading { version: String, percent: u8 },
    Installing(String),
    Done,
    Installed(PathBuf),
    Available(String),
}

pub fn download_ahead(enabled: bool) {
    AHEAD.store(enabled, Ordering::Relaxed);
}

pub fn start(updates: Updates, notify: impl Fn(UpdateStatus) + Send + 'static) -> UpdateStatus {
    let Ok(exe) = env::current_exe() else {
        return UpdateStatus::Done;
    };
    swap::remove_leftover(&exe);
    download_ahead(updates.downloads_ahead());
    let Some(asset) = ASSET.filter(|_| allowed()) else {
        return UpdateStatus::Done;
    };
    let Some(route) = Route::of(&exe) else {
        return UpdateStatus::Done;
    };
    let wanted = updates.auto || flagged(INSTALL_FLAG);
    let installs = route == Route::Swap && wanted && !flagged(UPDATED_FLAG);
    let spawned = thread::Builder::new()
        .name("trame-update".into())
        .spawn(move || follow(route, installs, asset, exe, &notify));
    match spawned {
        Ok(_) if installs => UpdateStatus::Checking,
        Ok(_) => UpdateStatus::Done,
        Err(error) => {
            eprintln!("trame: the update check did not start: {error}");
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

fn follow(route: Route, installs: bool, asset: &str, exe: PathBuf, notify: &dyn Fn(UpdateStatus)) {
    staged::tidy(&exe);
    if installs {
        let status = check(asset, &exe, notify);
        let installed = matches!(status, UpdateStatus::Installed(_));
        notify(status);
        if installed {
            return;
        }
    }
    watch::watch(route, &exe, asset, notify);
}

fn check(asset: &str, exe: &Path, notify: &dyn Fn(UpdateStatus)) -> UpdateStatus {
    let Some(latest) = staged::newest(exe).or_else(watch::newer_release) else {
        return UpdateStatus::Done;
    };
    match install(latest, asset, exe, notify) {
        Ok(()) => UpdateStatus::Installed(exe.to_path_buf()),
        Err(error) => {
            eprintln!("trame: the update to {latest} failed: {error}");
            UpdateStatus::Done
        }
    }
}

fn install(
    version: Version,
    asset: &str,
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
    let ready = staged::path(exe, version);
    let staged = swap::check_executable(&ready).is_ok();
    let (fresh, fetched) = match staged {
        true => (ready, Ok(())),
        false => {
            downloading(0);
            let fresh = swap::sibling(exe, NEW_SUFFIX);
            let fetched = fetch::download(version, asset, &fresh, &downloading);
            (fresh, fetched)
        }
    };
    let installed = fetched
        .inspect(|_| notify(UpdateStatus::Installing(number.clone())))
        .and_then(|()| swap::check_executable(&fresh))
        .and_then(|()| swap::replace(exe, &fresh));
    if installed.is_err() && fresh.exists() {
        if let Err(error) = fs::remove_file(&fresh) {
            eprintln!("trame: the partial download stays on disk: {error}");
        }
    }
    installed
}

fn ahead() -> bool {
    AHEAD.load(Ordering::Relaxed)
}
