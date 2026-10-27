use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::super::download::{self, Progress};
use super::super::route::Route;
use super::super::version::Version;
use super::{bundle, check, swap};

const READY_MARK: &str = ".ready";
const PART_SUFFIX: &str = ".part";

pub fn path(target: &Path, version: Version) -> PathBuf {
    swap::sibling(target, &format!(".{}{READY_MARK}", version.number()))
}

pub fn newest(target: &Path) -> Option<Version> {
    let current = Version::current()?;
    let newer = packages(target).into_iter().filter_map(|(version, path)| {
        let ready = version.is_some_and(|version| version > current);
        let whole = ready && check::release(&path).is_ok();
        whole.then_some(version).flatten()
    });
    newer.max()
}

pub fn fetch(route: Route, target: &Path, version: Version, shown: Progress) -> io::Result<()> {
    let ready = path(target, version);
    if check::release(&ready).is_ok() {
        return Ok(());
    }
    clear(target);
    if let Some(shown) = shown {
        shown(0);
    }
    let part = swap::sibling(&ready, PART_SUFFIX);
    let asset = route.asset(&version.number());
    let fetched = match route {
        Route::Bundle => bundle::fetch(version, &asset, &part, shown),
        _ => download::download(version, &asset, &part, shown),
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
        eprintln!("trame: an early update download stays on disk: {error}");
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Scratch(PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str) -> (Scratch, PathBuf) {
        let dir = std::env::temp_dir().join(format!("trame-staged-{name}-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("scratch folder");
        let exe = dir.join("trame.exe");
        fs::write(&exe, b"running").expect("the running program");
        (Scratch(dir), exe)
    }

    fn program() -> Vec<u8> {
        let mut bytes = b"MZ".to_vec();
        bytes.resize(1024 * 1024, 0);
        bytes
    }

    fn version(text: &str) -> Version {
        Version::parse(text).expect(text)
    }

    #[test]
    fn a_whole_newer_package_waits_for_the_next_launch() {
        let (_scratch, exe) = scratch("newest");
        fs::write(path(&exe, version("999.0.0")), program()).expect("a staged package");
        fs::write(path(&exe, version("999.1.0")), b"MZ cut short").expect("a broken one");
        assert_eq!(newest(&exe), Some(version("999.0.0")));
    }

    #[test]
    fn tidying_drops_old_and_partial_packages_but_keeps_newer_ones() {
        let (_scratch, exe) = scratch("tidy");
        let old = path(&exe, version("0.0.1"));
        let newer = path(&exe, version("999.0.0"));
        let partial = swap::sibling(&path(&exe, version("999.1.0")), PART_SUFFIX);
        let archive = swap::sibling(&partial, ".zip");
        for file in [&old, &newer, &partial, &archive] {
            fs::write(file, program()).expect("a package");
        }
        let unpacked = swap::sibling(&partial, ".unzip");
        fs::create_dir_all(unpacked.join("trame.app")).expect("an unpacked folder");
        tidy(&exe);
        assert!(!old.exists() && !partial.exists());
        assert!(!archive.exists() && !unpacked.exists());
        assert!(newer.exists() && exe.exists());
    }
}
