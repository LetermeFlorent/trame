use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::log_error;

use super::super::download::Progress;
use super::super::route::Route;
use super::super::version::Version;
use super::{bundle, check, signed, swap};

const READY_MARK: &str = ".ready";
pub const PART_SUFFIX: &str = ".part";

pub fn path(target: &Path, version: Version) -> PathBuf {
    swap::sibling(target, &format!(".{}{READY_MARK}", version.number()))
}

pub fn fetch(route: Route, target: &Path, version: Version, shown: Progress) -> io::Result<()> {
    let ready = path(target, version);
    if check::release(&ready).is_ok() {
        return Ok(());
    }
    clear(target);
    shown(0);
    let part = swap::sibling(&ready, PART_SUFFIX);
    let asset = route.asset(&version.number());
    let fetched = match route {
        Route::Bundle => bundle::fetch(version, &asset, &part, shown),
        _ => signed::fetch(version, &asset, &part, shown).map(drop),
    };
    let placed = fetched
        .and_then(|()| check::release(&part))
        .and_then(|()| fs::rename(&part, &ready));
    if placed.is_err() {
        swap::discard_partial(&part, "download");
    }
    placed
}

pub fn tidy(target: &Path) {
    let current = Version::current();
    for (version, path) in packages(target) {
        let outdated = version.is_none_or(|version| Some(version) <= current);
        if outdated {
            remove(&path);
        }
    }
}

fn clear(target: &Path) {
    for (_, path) in packages(target) {
        remove(&path);
    }
}

fn remove(path: &Path) {
    if let Err(error) = swap::remove(path) {
        log_error!("an early update download stays on disk: {error}");
    }
}

fn packages(target: &Path) -> Vec<(Option<Version>, PathBuf)> {
    let prefix = target.file_name().and_then(|name| name.to_str());
    let (Some(folder), Some(prefix)) = (target.parent(), prefix) else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let rest = name.to_str()?.strip_prefix(prefix)?.strip_prefix('.')?;
            let (number, tail) = rest.split_once(READY_MARK)?;
            let version = tail.is_empty().then(|| Version::parse(number)).flatten();
            Some((version, entry.path()))
        })
        .collect()
}
