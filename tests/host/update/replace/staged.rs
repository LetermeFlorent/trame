use std::fs;
use std::path::PathBuf;

use trame::host::update::replace::staged::{PART_SUFFIX, path, tidy};
use trame::host::update::replace::swap;

use crate::update::version;

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
