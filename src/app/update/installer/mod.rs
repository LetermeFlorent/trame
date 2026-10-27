mod package;
mod setup;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::download;
use super::replace::{check, swap};
use super::route::Route;
use super::version::Version;

pub use package::Package;

const FOLDER: &str = "trame-update";
const PART_SUFFIX: &str = ".part";

pub fn fetch(route: Route, version: Version) -> io::Result<()> {
    let number = version.number();
    let target = path(route, &number)?;
    if check::release(&target).is_ok() {
        return Ok(());
    }
    clear(route);
    if let Some(folder) = target.parent() {
        fs::create_dir_all(folder)?;
    }
    let part = swap::sibling(&target, PART_SUFFIX);
    let fetched = download::download(version, &route.asset(&number), &part, None)
        .and_then(|()| check::release(&part))
        .and_then(|()| fs::rename(&part, &target));
    if fetched.is_err() {
        swap::discard_partial(&part, "installer");
    }
    fetched
}

pub fn run(route: Route, number: &str, exe: &Path) -> io::Result<()> {
    let file = path(route, number)?;
    match route {
        Route::Package(package) => package.install(&file, exe),
        _ => setup::run(&file),
    }
}

pub fn clear(route: Route) {
    let Some(folder) = folder(route).filter(|folder| folder.exists()) else {
        return;
    };
    if let Err(error) = fs::remove_dir_all(&folder) {
        eprintln!("trame: an old installer stays on disk: {error}");
    }
}

fn folder(route: Route) -> Option<PathBuf> {
    match route {
        Route::Package(_) => package::folder(),
        _ => Some(env::temp_dir().join(FOLDER)),
    }
}

fn path(route: Route, number: &str) -> io::Result<PathBuf> {
    folder(route)
        .map(|folder| folder.join(route.asset(number)))
        .ok_or_else(|| io::Error::other("no folder to keep the installer in"))
}
