mod package;
mod setup;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::replace::signed::{self, SignedList};
use super::replace::{check, swap};
use super::route::Route;
use super::version::Version;

pub use package::Package;

const FOLDER: &str = "trame-update";
const PART_SUFFIX: &str = ".part";

pub fn fetch(route: Route, version: Version) -> io::Result<()> {
    let folder = folder(route).ok_or_else(no_folder)?;
    let asset = route.asset(&version.number());
    let target = folder.join(&asset);
    if ready(&folder, &asset, version).is_ok() {
        return Ok(());
    }
    clear(route);
    fs::create_dir_all(&folder)?;
    let part = swap::sibling(&target, PART_SUFFIX);
    let fetched = signed::fetch(version, &asset, &part, None)
        .and_then(|signed| check::release(&part).and_then(|()| signed.save(&folder)))
        .and_then(|()| fs::rename(&part, &target));
    if fetched.is_err() {
        swap::discard_partial(&part, "installer");
    }
    fetched
}

pub fn run(route: Route, number: &str, exe: &Path) -> io::Result<()> {
    let version =
        Version::parse(number).ok_or_else(|| io::Error::other("no version to install"))?;
    let folder = folder(route).ok_or_else(no_folder)?;
    let asset = route.asset(number);
    let expected = SignedList::load(&folder, version)?.listed(&asset)?;
    let file = folder.join(asset);
    match route {
        Route::Package(package) => package.install(&file, &expected, exe),
        _ => setup::run(&file, &expected),
    }
}

fn ready(folder: &Path, asset: &str, version: Version) -> io::Result<()> {
    let file = folder.join(asset);
    check::release(&file)?;
    SignedList::load(folder, version)?.check(asset, &file)
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

fn no_folder() -> io::Error {
    io::Error::other("no folder to keep the installer in")
}
