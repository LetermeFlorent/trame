use std::fs;
use std::path::PathBuf;

use trame::host::update::replace::check::{BUNDLE_PROGRAM, MAGICS, SMALLEST_RELEASE, release};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("trame-check-{name}-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("scratch folder");
        Self(dir)
    }

    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().expect("a folder")).expect("its folder");
        fs::write(&path, bytes).expect("scratch file");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn release_file(magic: &[u8]) -> Vec<u8> {
    let mut bytes = magic.to_vec();
    bytes.resize(SMALLEST_RELEASE as usize, 0);
    bytes
}

#[test]
fn every_kind_of_release_is_accepted_when_whole() {
    let scratch = Scratch::new("kinds");
    for (index, magic) in MAGICS.iter().enumerate() {
        let path = scratch.file(&format!("release-{index}"), &release_file(magic));
        assert!(release(&path).is_ok(), "{magic:?}");
    }
}

#[test]
fn a_page_a_short_file_or_nothing_is_refused() {
    let scratch = Scratch::new("refused");
    assert!(release(&scratch.file("small", b"MZ tiny")).is_err());
    assert!(release(&scratch.file("page", &release_file(b"<html>"))).is_err());
    assert!(release(&scratch.file("empty", b"")).is_err());
    assert!(release(&scratch.0.join("missing")).is_err());
}

#[test]
fn an_application_folder_is_judged_by_its_program() {
    let scratch = Scratch::new("bundle");
    let program = BUNDLE_PROGRAM.join("/");
    scratch.file(
        &format!("whole.app/{program}"),
        &release_file(b"\xca\xfe\xba\xbe"),
    );
    scratch.file(&format!("empty.app/{program}"), b"");
    assert!(release(&scratch.0.join("whole.app")).is_ok());
    assert!(release(&scratch.0.join("empty.app")).is_err());
}
