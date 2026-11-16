#[cfg(target_os = "macos")]
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Command;

use trame::host::update::replace::bundle::root;
#[cfg(target_os = "macos")]
use trame::host::update::replace::bundle::{DITTO, UNPACK_SUFFIX, unpack};
#[cfg(target_os = "macos")]
use trame::host::update::replace::{check, swap};

#[test]
fn the_application_folder_is_found_from_its_program() {
    let exe = Path::new("/Applications/trame.app/Contents/MacOS/trame");
    assert_eq!(root(exe), Some(PathBuf::from("/Applications/trame.app")));
}

#[test]
fn a_program_outside_an_application_folder_has_none() {
    assert_eq!(root(Path::new("/usr/bin/trame")), None);
    assert_eq!(root(Path::new("/opt/trame/Contents/MacOS/trame")), None);
    assert_eq!(root(Path::new("C:/Programs/trame/trame.exe")), None);
}

#[cfg(target_os = "macos")]
#[test]
fn an_archive_made_by_the_release_opens_into_the_new_application() {
    let scratch = std::env::temp_dir().join(format!("trame-bundle-{}", std::process::id()));
    let program = scratch.join("build/trame.app/Contents/MacOS/trame");
    fs::create_dir_all(program.parent().expect("a folder")).expect("the build");
    let mut bytes = b"\xca\xfe\xba\xbe".to_vec();
    bytes.resize(1024 * 1024, 0);
    fs::write(&program, bytes).expect("the program");
    let archive = scratch.join("trame-macos.zip");
    let zipped = Command::new(DITTO)
        .args(["-c", "-k", "--keepParent"])
        .arg(scratch.join("build/trame.app"))
        .arg(&archive)
        .status()
        .expect("ditto runs");
    assert!(zipped.success());
    let part = scratch.join("trame.app.9.9.9.ready.part");
    unpack(&archive, &part).expect("the archive opens");
    assert!(check::release(&part).is_ok());
    assert!(!swap::sibling(&part, UNPACK_SUFFIX).exists());
    fs::remove_dir_all(&scratch).expect("cleanup");
}
