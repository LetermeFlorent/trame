mod download;
mod installer;
mod launch;
mod replace;
mod route;
mod version;
mod watch;

use std::ffi::OsString;

use trame::host::update::version::Version;
use trame::host::update::{INSTALL_FLAG, UPDATED_FLAG, installs, plain};

pub fn arguments(list: &[&str]) -> impl Iterator<Item = OsString> {
    list.iter()
        .map(OsString::from)
        .collect::<Vec<_>>()
        .into_iter()
}

pub fn version(text: &str) -> Version {
    Version::parse(text).expect(text)
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

#[test]
fn only_a_start_without_an_update_flag_is_plain() {
    assert!(plain(arguments(&["C:/work"])));
    assert!(!plain(arguments(&[INSTALL_FLAG])));
    assert!(!plain(arguments(&[UPDATED_FLAG])));
}
