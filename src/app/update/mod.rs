mod fetch;
mod swap;
mod version;

use std::env;
use std::fs;
use std::io;
use std::thread::{self, JoinHandle};

use version::Version;

const RELEASES: &str = "https://github.com/LetermeFlorent/trame/releases";
const SKIP_VARIABLE: &str = "TRAME_NO_UPDATE";
const UPDATED_FLAG: &str = "--updated";
const NEW_SUFFIX: &str = ".new";

#[cfg(windows)]
const ASSET: Option<&str> = Some("trame.exe");
#[cfg(not(windows))]
const ASSET: Option<&str> = None;

pub struct Update {
    check: Option<JoinHandle<Option<Version>>>,
}

impl Update {
    pub fn start() -> Self {
        if let Ok(exe) = env::current_exe() {
            swap::remove_leftover(&exe);
        }
        let check = enabled()
            .then(|| {
                thread::Builder::new()
                    .name("trame-update".into())
                    .spawn(fetch::latest)
                    .ok()
            })
            .flatten();
        Self { check }
    }

    pub fn relaunched(self) -> bool {
        let latest = self.check.and_then(|check| check.join().ok()).flatten();
        let (Some(latest), Some(asset)) = (latest, ASSET) else {
            return false;
        };
        if Version::current().is_none_or(|current| latest <= current) {
            return false;
        }
        match install(latest, asset) {
            Ok(()) => true,
            Err(error) => {
                eprintln!("trame: the update to {latest} failed: {error}");
                false
            }
        }
    }
}

fn enabled() -> bool {
    ASSET.is_some()
        && !cfg!(debug_assertions)
        && env::var_os(SKIP_VARIABLE).is_none()
        && !env::args_os().any(|arg| arg == UPDATED_FLAG)
}

fn install(version: Version, asset: &str) -> io::Result<()> {
    let exe = env::current_exe()?;
    let fresh = swap::sibling(&exe, NEW_SUFFIX);
    let installed = fetch::download(version, asset, &fresh)
        .and_then(|()| swap::check_executable(&fresh))
        .and_then(|()| swap::replace(&exe, &fresh));
    if installed.is_err() && fresh.exists() {
        if let Err(error) = fs::remove_file(&fresh) {
            eprintln!("trame: the partial download stays on disk: {error}");
        }
    }
    installed?;
    swap::relaunch(&exe)
}
