use std::fs::File;
use std::io;
use std::path::Path;
use std::process::Command;

use super::super::replace::signed;

const ARGUMENTS: [&str; 5] = [
    "/SILENT",
    "/SUPPRESSMSGBOXES",
    "/NORESTART",
    "/ALLUSERS",
    "/RELAUNCH=yes",
];

pub fn run(installer: &Path, expected: &str) -> io::Result<()> {
    let locked = lock(installer)?;
    signed::confirm(expected, &locked)?;
    let started = Command::new(installer).args(ARGUMENTS).spawn().map(drop);
    drop(locked);
    started
}

#[cfg(windows)]
fn lock(path: &Path) -> io::Result<File> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
    OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
}

#[cfg(not(windows))]
fn lock(path: &Path) -> io::Result<File> {
    File::open(path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::super::super::route::Route;
    use super::*;

    const SCRIPT: &str = include_str!("../../../../packaging/windows/trame.iss");

    #[test]
    fn the_download_is_the_installer_the_release_publishes() {
        assert_eq!(Route::Setup.asset("0.3.6"), "trame-0.3.6-setup.exe");
        assert!(SCRIPT.contains("OutputBaseFilename=trame-{#AppVersion}-setup"));
    }

    #[test]
    fn the_installer_accepts_every_user_from_the_command_line() {
        let overrides = SCRIPT
            .lines()
            .find_map(|line| line.strip_prefix("PrivilegesRequiredOverridesAllowed="))
            .expect("the install mode can be overridden");
        assert!(
            overrides
                .split_whitespace()
                .any(|mode| mode == "commandline" || mode == "dialog")
        );
        assert!(ARGUMENTS.contains(&"/ALLUSERS"));
    }

    #[test]
    fn the_installer_reopens_trame_when_asked_to() {
        assert!(SCRIPT.contains("{param:relaunch|no}"));
        assert!(ARGUMENTS.contains(&"/RELAUNCH=yes"));
    }

    #[test]
    fn a_setup_that_does_not_match_its_checksum_never_starts() {
        let folder = scratch("mismatch");
        let setup = folder.join("trame-9.9.9-setup.exe");
        fs::write(&setup, b"MZ not the signed setup").expect("a setup");
        let refused = run(&setup, &"0".repeat(128));
        fs::remove_dir_all(&folder).expect("cleanup");
        assert_eq!(
            refused.map_err(|error| error.kind()),
            Err(io::ErrorKind::InvalidData)
        );
    }

    #[cfg(windows)]
    #[test]
    fn the_checked_setup_cannot_change_until_it_has_started() {
        let folder = scratch("locked");
        let setup = folder.join("trame-9.9.9-setup.exe");
        fs::write(&setup, b"MZ setup").expect("a setup");
        let locked = lock(&setup).expect("the setup opens");
        let written = fs::OpenOptions::new().write(true).open(&setup).is_ok();
        let moved = fs::rename(&setup, folder.join("moved.exe")).is_ok();
        let removed = fs::remove_file(&setup).is_ok();
        drop(locked);
        fs::remove_dir_all(&folder).expect("cleanup");
        assert!(!written && !moved && !removed);
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let folder =
            std::env::temp_dir().join(format!("trame-setup-{name}-{}", std::process::id()));
        fs::create_dir_all(&folder).expect("scratch folder");
        folder
    }
}
