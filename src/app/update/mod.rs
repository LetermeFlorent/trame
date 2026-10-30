mod download;
mod installer;
mod launch;
mod replace;
mod route;
mod version;
mod watch;

use std::env;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;

pub use launch::{Handoff, restart, start_again};
use replace::{bundle, staged, swap};
use route::Route;
use version::Version;

const RELEASES: &str = "https://github.com/LetermeFlorent/trame/releases";
const SKIP_VARIABLE: &str = "TRAME_NO_UPDATE";
const UPDATED_FLAG: &str = "--updated";
const INSTALL_FLAG: &str = "--install";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    Checking,
    Downloading { version: String, percent: u8 },
    Installing(String),
    Done,
    Installed(Handoff),
    Available(String),
}

pub fn start(notify: impl Fn(UpdateStatus) + Send + 'static) -> UpdateStatus {
    let Ok(exe) = env::current_exe() else {
        return UpdateStatus::Done;
    };
    swap::remove_leftover(&bundle::root(&exe).unwrap_or_else(|| exe.clone()));
    let Some(route) = allowed().then(|| Route::of(&exe)).flatten() else {
        return UpdateStatus::Done;
    };
    let installs = installs(env::args_os());
    let spawned = thread::Builder::new()
        .name("trame-update".into())
        .spawn(move || follow(route, installs, exe, &notify));
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

fn installs(arguments: impl Iterator<Item = OsString>) -> bool {
    let arguments: Vec<OsString> = arguments.collect();
    let flagged = |flag: &str| arguments.iter().any(|arg| arg == flag);
    flagged(INSTALL_FLAG) && !flagged(UPDATED_FLAG)
}

fn follow(route: Route, installs: bool, exe: PathBuf, notify: &dyn Fn(UpdateStatus)) {
    staged::tidy(&route.target(&exe));
    installer::clear(route);
    if installs {
        notify(check(route, &exe, notify));
    }
    watch::watch(notify);
}

fn check(route: Route, exe: &Path, notify: &dyn Fn(UpdateStatus)) -> UpdateStatus {
    let Some(latest) = watch::newer_release() else {
        return UpdateStatus::Done;
    };
    match install(route, latest, exe, notify) {
        Ok(()) => UpdateStatus::Installed(Handoff::after(route, exe, latest)),
        Err(error) => {
            eprintln!("trame: the update to {latest} failed: {error}");
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
    match route.replaces() {
        true => replace::put_in_place(route, exe, version, &downloading, &installing),
        false => installer::fetch(route, version, &downloading).map(|()| installing()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(list: &[&str]) -> impl Iterator<Item = OsString> {
        list.iter()
            .map(OsString::from)
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn a_start_asked_to_install_shows_the_loader() {
        assert!(installs(arguments(&["C:/work", INSTALL_FLAG])));
    }

    #[test]
    fn a_start_after_an_update_never_installs_again() {
        assert!(!installs(arguments(&[INSTALL_FLAG, UPDATED_FLAG])));
        assert!(!installs(arguments(&[UPDATED_FLAG])));
    }

    #[test]
    fn a_plain_start_installs_nothing() {
        assert!(!installs(arguments(&["C:/work"])));
    }
}
