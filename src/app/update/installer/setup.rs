use std::io;
use std::path::Path;
use std::process::Command;

const ARGUMENTS: [&str; 5] = [
    "/SILENT",
    "/SUPPRESSMSGBOXES",
    "/NORESTART",
    "/ALLUSERS",
    "/RELAUNCH=yes",
];

pub fn run(installer: &Path) -> io::Result<()> {
    Command::new(installer).args(ARGUMENTS).spawn().map(drop)
}

#[cfg(test)]
mod tests {
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
}
