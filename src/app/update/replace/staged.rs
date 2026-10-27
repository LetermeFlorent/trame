use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::version::Version;
use super::{fetch, swap};

const READY_SUFFIX: &str = ".ready";
const PART_SUFFIX: &str = ".part";

pub fn path(exe: &Path, version: Version) -> PathBuf {
    swap::sibling(exe, &format!(".{}{READY_SUFFIX}", version.number()))
}

pub fn newest(exe: &Path) -> Option<Version> {
    let current = Version::current()?;
    let newer = packages(exe).into_iter().filter_map(|(version, path)| {
        let ready = version.is_some_and(|version| version > current);
        let whole = ready && swap::check_executable(&path).is_ok();
        whole.then_some(version).flatten()
    });
    newer.max()
}

pub fn fetch(exe: &Path, version: Version, asset: &str) -> io::Result<()> {
    let target = path(exe, version);
    if swap::check_executable(&target).is_ok() {
        return Ok(());
    }
    clear(exe);
    let part = swap::sibling(&target, PART_SUFFIX);
    let fetched = fetch::download_quietly(version, asset, &part)
        .and_then(|()| swap::check_executable(&part))
        .and_then(|()| fs::rename(&part, &target));
    if fetched.is_err() && part.exists() {
        remove(&part);
    }
    fetched
}

pub fn tidy(exe: &Path) {
    let current = Version::current();
    for (version, path) in packages(exe) {
        let outdated = version.is_none_or(|version| Some(version) <= current);
        if outdated {
            remove(&path);
        }
    }
}

fn clear(exe: &Path) {
    for (_, path) in packages(exe) {
        remove(&path);
    }
}

fn remove(path: &Path) {
    if let Err(error) = fs::remove_file(path) {
        eprintln!("trame: an early update download stays on disk: {error}");
    }
}

fn packages(exe: &Path) -> Vec<(Option<Version>, PathBuf)> {
    let prefix = exe.file_name().and_then(|name| name.to_str());
    let (Some(folder), Some(prefix)) = (exe.parent(), prefix) else {
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
            let ready = rest.strip_suffix(READY_SUFFIX);
            let partial = rest
                .strip_suffix(PART_SUFFIX)
                .and_then(|rest| rest.strip_suffix(READY_SUFFIX));
            let number = ready.or(partial)?;
            let version = ready.and_then(|_| Version::parse(number));
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
        for file in [&old, &newer, &partial] {
            fs::write(file, program()).expect("a package");
        }
        tidy(&exe);
        assert!(!old.exists() && !partial.exists());
        assert!(newer.exists() && exe.exists());
    }
}
