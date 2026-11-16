use std::ffi::OsString;
use std::path::{Path, PathBuf};

use trame::host::update::installer::Package;
use trame::host::update::launch::{Handoff, Next, next_arguments};
use trame::host::update::route::Route;
use trame::host::update::version::Version;
use trame::host::update::{INSTALL_FLAG, UPDATED_FLAG};

use crate::update::arguments;

fn handoff(route: Route) -> Next {
    let version = Version::parse("9.8.7").expect("a version");
    Handoff::after(route, Path::new("trame.exe"), version).0
}

#[test]
fn a_relaunch_after_an_install_carries_the_flag_once() {
    let next = next_arguments(arguments(&["C:/work", UPDATED_FLAG]), Some(UPDATED_FLAG));
    assert_eq!(next, ["C:/work", UPDATED_FLAG].map(OsString::from));
}

#[test]
fn a_restart_for_an_update_asks_the_new_start_to_install() {
    let current = arguments(&[UPDATED_FLAG, "C:/work", INSTALL_FLAG]);
    let next = next_arguments(current, Some(INSTALL_FLAG));
    assert_eq!(next, ["C:/work", INSTALL_FLAG].map(OsString::from));
}

#[test]
fn a_restart_for_an_update_keeps_every_other_argument_in_order() {
    let current = arguments(&["--one", "C:/work", "--two"]);
    let next = next_arguments(current, Some(INSTALL_FLAG));
    assert_eq!(
        next,
        ["--one", "C:/work", "--two", INSTALL_FLAG].map(OsString::from)
    );
}

#[test]
fn starting_again_drops_both_flags() {
    let next = next_arguments(arguments(&[INSTALL_FLAG, "C:/work", UPDATED_FLAG]), None);
    assert_eq!(next, [OsString::from("C:/work")]);
}

#[test]
fn a_swapped_program_starts_again_by_itself() {
    let exe = PathBuf::from("trame.exe");
    assert_eq!(handoff(Route::Swap), Next::Relaunch(exe.clone()));
    assert_eq!(handoff(Route::Bundle), Next::Relaunch(exe));
}

#[test]
fn an_installed_copy_hands_over_to_the_installer_it_downloaded() {
    let exe = PathBuf::from("trame.exe");
    for route in [
        Route::Setup,
        Route::Package(Package::Deb),
        Route::Package(Package::Rpm),
    ] {
        let expected = Next::Installer(route, "9.8.7".to_owned(), exe.clone());
        assert_eq!(handoff(route), expected);
    }
}
