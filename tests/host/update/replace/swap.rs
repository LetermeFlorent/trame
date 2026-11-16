use std::fs;
use std::path::PathBuf;

use trame::host::update::replace::swap::{OLD_SUFFIX, replace, sibling};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("trame-swap-{name}-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("scratch folder");
        Self(dir)
    }

    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).expect("scratch file");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_new_version_takes_the_place_and_the_old_one_is_kept_aside() {
    let scratch = Scratch::new("replace");
    let exe = scratch.file("trame.exe", b"running");
    let fresh = scratch.file("trame.exe.new", b"fresh");
    scratch.file("trame.exe.old", b"older");
    replace(&exe, &fresh).expect("the swap");
    assert_eq!(fs::read(&exe).unwrap(), b"fresh");
    assert_eq!(fs::read(sibling(&exe, OLD_SUFFIX)).unwrap(), b"running");
    assert!(!fresh.exists());
}

#[test]
fn a_whole_application_folder_is_swapped_the_same_way() {
    let scratch = Scratch::new("bundle");
    let app = scratch.0.join("trame.app");
    let fresh = scratch.0.join("trame.app.9.9.9.ready");
    let older = sibling(&app, OLD_SUFFIX);
    for (folder, text) in [(&app, "running"), (&fresh, "fresh"), (&older, "older")] {
        fs::create_dir_all(folder).expect("an application folder");
        fs::write(folder.join("version"), text).expect("its content");
    }
    replace(&app, &fresh).expect("the swap");
    assert_eq!(fs::read(app.join("version")).unwrap(), b"fresh");
    assert_eq!(fs::read(older.join("version")).unwrap(), b"running");
    assert!(!fresh.exists());
}

#[test]
fn a_failed_swap_puts_the_running_version_back() {
    let scratch = Scratch::new("rollback");
    let exe = scratch.file("trame.exe", b"running");
    let missing = scratch.0.join("trame.exe.new");
    assert!(replace(&exe, &missing).is_err());
    assert_eq!(fs::read(&exe).unwrap(), b"running");
}
