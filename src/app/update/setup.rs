use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use super::version::Version;
use super::{fetch, swap};

const FOLDER: &str = "trame-update";
const PART_SUFFIX: &str = ".part";
const ARGUMENTS: [&str; 5] = [
    "/SILENT",
    "/SUPPRESSMSGBOXES",
    "/NORESTART",
    "/ALLUSERS",
    "/RELAUNCH=yes",
];

pub fn fetch(version: Version) -> io::Result<()> {
    let number = version.number();
    let target = path(&number);
    if swap::check_executable(&target).is_ok() {
        return Ok(());
    }
    clear();
    fs::create_dir_all(folder())?;
    let part = swap::sibling(&target, PART_SUFFIX);
    let fetched = fetch::download_quietly(version, &asset(&number), &part)
        .and_then(|()| swap::check_executable(&part))
        .and_then(|()| fs::rename(&part, &target));
    if fetched.is_err() {
        swap::discard_partial(&part, "installer");
    }
    fetched
}

pub fn run(number: &str) -> io::Result<()> {
    Command::new(path(number)).args(ARGUMENTS).spawn().map(drop)
}

pub fn clear() {
    let folder = folder();
    if folder.exists() {
        if let Err(error) = fs::remove_dir_all(&folder) {
            eprintln!("trame: an old installer stays on disk: {error}");
        }
    }
}

fn folder() -> PathBuf {
    env::temp_dir().join(FOLDER)
}

fn path(number: &str) -> PathBuf {
    folder().join(asset(number))
}

fn asset(number: &str) -> String {
    format!("trame-{number}-setup.exe")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT: &str = include_str!("../../../packaging/windows/trame.iss");

    #[test]
    fn the_download_is_the_installer_the_release_publishes() {
        assert_eq!(asset("0.3.6"), "trame-0.3.6-setup.exe");
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
}
