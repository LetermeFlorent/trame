use std::ffi::OsStr;

use super::*;

#[test]
fn the_user_curl_settings_are_never_read() {
    assert_eq!(curl().get_args().next(), Some(OsStr::new("-q")));
}

#[cfg(not(windows))]
#[test]
fn curl_is_the_copy_the_system_installs() {
    assert_eq!(curl_program(), Path::new("/usr/bin/curl"));
}

#[test]
fn a_release_file_comes_from_the_folder_of_its_version() {
    let version = Version::parse("1.2.3").expect("a version");
    assert_eq!(
        address(version, "SHA512SUMS"),
        format!("{RELEASES}/download/v1.2.3/SHA512SUMS")
    );
}
