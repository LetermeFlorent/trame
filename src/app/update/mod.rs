mod fetch;
mod swap;
mod version;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;

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
    Downloading(String),
    Done,
    Installed(PathBuf),
}

pub fn start(notify: impl Fn(UpdateStatus) + Send + 'static) -> UpdateStatus {
    let Ok(exe) = env::current_exe() else {
        return UpdateStatus::Done;
    };
    swap::remove_leftover(&exe);
    let Some(asset) = ASSET.filter(|_| enabled()) else {
        return UpdateStatus::Done;
    };
    let spawned = thread::Builder::new()
        .name("trame-update".into())
        .spawn(move || notify(check(asset, exe, &notify)));
    match spawned {
        Ok(_) => UpdateStatus::Checking,
        Err(error) => {
            eprintln!("trame: the update check did not start: {error}");
            UpdateStatus::Done
        }
    }
}

pub fn relaunch(exe: &Path) -> io::Result<()> {
    swap::relaunch(exe)
}

fn enabled() -> bool {
    !cfg!(debug_assertions)
        && env::var_os(SKIP_VARIABLE).is_none()
        && !env::args_os().any(|arg| arg == UPDATED_FLAG)
}

fn check(asset: &str, exe: PathBuf, notify: &dyn Fn(UpdateStatus)) -> UpdateStatus {
    let Some(latest) = fetch::latest() else {
        return UpdateStatus::Done;
    };
    if Version::current().is_none_or(|current| latest <= current) {
        return UpdateStatus::Done;
    }
    notify(UpdateStatus::Downloading(latest.number()));
    match install(latest, asset, &exe) {
        Ok(()) => UpdateStatus::Installed(exe),
        Err(error) => {
            eprintln!("trame: the update to {latest} failed: {error}");
            UpdateStatus::Done
        }
    }
}

fn install(version: Version, asset: &str, exe: &Path) -> io::Result<()> {
    let fresh = swap::sibling(exe, NEW_SUFFIX);
    let installed = fetch::download(version, asset, &fresh)
        .and_then(|()| swap::check_executable(&fresh))
        .and_then(|()| swap::replace(exe, &fresh));
    if installed.is_err() && fresh.exists() {
        if let Err(error) = fs::remove_file(&fresh) {
            eprintln!("trame: the partial download stays on disk: {error}");
        }
    }
    installed
}
