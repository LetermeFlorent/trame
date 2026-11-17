use std::fs;
use std::path::Path;

use trame::host::update::installer::Package;
use trame::host::update::replace::swap;
use trame::host::update::route::{PROBE_SUFFIX, Route, writable};

const WORKFLOW: &str = include_str!("../../../.github/workflows/release.yml");

#[test]
fn a_folder_trame_can_write_swaps_the_executable_in_place() {
    assert_eq!(Route::pick(true, false), Some(Route::Swap));
    assert_eq!(Route::pick(true, true), Some(Route::Swap));
}

#[test]
fn an_install_for_every_user_goes_through_the_installer() {
    assert_eq!(Route::pick(false, true), Some(Route::Setup));
}

#[test]
fn a_copy_in_a_locked_folder_outside_the_installer_is_left_alone() {
    assert_eq!(Route::pick(false, false), None);
}

#[test]
fn the_probe_leaves_nothing_behind() {
    let folder = std::env::temp_dir().join(format!("trame-route-{}", std::process::id()));
    fs::create_dir_all(&folder).expect("scratch folder");
    let exe = folder.join("trame.exe");
    assert!(writable(&exe));
    assert!(!swap::sibling(&exe, PROBE_SUFFIX).exists());
    fs::remove_dir_all(&folder).expect("cleanup");
}

#[test]
fn a_mac_update_swaps_the_whole_application() {
    let exe = Path::new("/Applications/trame.app/Contents/MacOS/trame");
    assert_eq!(
        Route::Bundle.target(exe),
        Path::new("/Applications/trame.app")
    );
    assert_eq!(Route::Swap.target(exe), exe);
    assert!(Route::Bundle.replaces() && Route::Swap.replaces());
    assert!(!Route::Setup.replaces() && !Route::Package(Package::Deb).replaces());
}

#[test]
fn each_route_downloads_a_file_the_release_publishes() {
    assert_eq!(Route::Swap.asset("0.4.0"), "trame.exe");
    assert_eq!(Route::Bundle.asset("0.4.0"), "trame-0.4.0-macos.zip");
    assert!(WORKFLOW.contains("dist/trame-$version-macos.zip"));
    assert!(WORKFLOW.contains("trame.exe"));
}
