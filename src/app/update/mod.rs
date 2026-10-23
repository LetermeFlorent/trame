mod fetch;
mod launch;
mod progress;
mod route;
mod setup;
mod swap;
mod version;
mod watch;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;

pub use launch::{relaunch, restart};
use route::Route;
use version::Version;

const RELEASES: &str = "https://github.com/LetermeFlorent/trame/releases";
const SKIP_VARIABLE: &str = "TRAME_NO_UPDATE";
const UPDATED_FLAG: &str = "--updated";
const NEW_SUFFIX: &str = ".new";

#[cfg(windows)]
const ASSET: Option<&str> = Some("trame.exe");
#[cfg(not(windows))]
const ASSET: Option<&str> = None;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    Checking,
    Downloading { version: String, percent: u8 },
    Installing(String),
    Done,
    Installed(PathBuf),
    Available(String),
}

pub fn start(notify: impl Fn(UpdateStatus) + Send + 'static) -> UpdateStatus {
    let Ok(exe) = env::current_exe() else {
        return UpdateStatus::Done;
    };
    swap::remove_leftover(&exe);
    let Some(asset) = ASSET.filter(|_| allowed()) else {
        return UpdateStatus::Done;
    };
    let Some(route) = Route::of(&exe) else {
        return UpdateStatus::Done;
    };
    let installs = route == Route::Swap && !just_updated();
    let spawned = thread::Builder::new()
        .name("trame-update".into())
        .spawn(move || follow(route, installs.then_some(asset), exe, &notify));
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

fn just_updated() -> bool {
    env::args_os().any(|arg| arg == UPDATED_FLAG)
}

fn follow(route: Route, asset: Option<&str>, exe: PathBuf, notify: &dyn Fn(UpdateStatus)) {
    if let Some(asset) = asset {
        let status = check(asset, exe, notify);
        let installed = matches!(status, UpdateStatus::Installed(_));
        notify(status);
        if installed {
            return;
        }
    }
    watch::watch(route, notify);
}

fn check(asset: &str, exe: PathBuf, notify: &dyn Fn(UpdateStatus)) -> UpdateStatus {
    let Some(latest) = watch::newer_release() else {
        return UpdateStatus::Done;
    };
    match install(latest, asset, &exe, notify) {
        Ok(()) => UpdateStatus::Installed(exe),
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
    downloading(0);
    let fresh = swap::sibling(exe, NEW_SUFFIX);
    let installed = fetch::download(version, asset, &fresh, &downloading)
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
