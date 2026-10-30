use std::env;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::route::Route;
use super::version::Version;
use super::{INSTALL_FLAG, UPDATED_FLAG, installer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handoff(Next);

#[derive(Clone, Debug, PartialEq, Eq)]
enum Next {
    Relaunch(PathBuf),
    Installer(Route, String, PathBuf),
}

impl Handoff {
    pub(super) fn after(route: Route, exe: &Path, version: Version) -> Self {
        let exe = exe.to_path_buf();
        Self(match route.replaces() {
            true => Next::Relaunch(exe),
            false => Next::Installer(route, version.number(), exe),
        })
    }

    pub fn start(&self) -> io::Result<()> {
        match &self.0 {
            Next::Relaunch(exe) => spawn_again(exe, Some(UPDATED_FLAG)),
            Next::Installer(route, number, exe) => installer::run(*route, number, exe),
        }
    }
}

pub fn restart() -> io::Result<()> {
    spawn_again(&env::current_exe()?, Some(INSTALL_FLAG))
}

pub fn start_again() -> io::Result<()> {
    spawn_again(&env::current_exe()?, None)
}

fn spawn_again(exe: &Path, flag: Option<&str>) -> io::Result<()> {
    Command::new(exe)
        .args(next_arguments(env::args_os().skip(1), flag))
        .spawn()
        .map(drop)
}

fn next_arguments(current: impl Iterator<Item = OsString>, flag: Option<&str>) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = current
        .filter(|arg| arg != UPDATED_FLAG && arg != INSTALL_FLAG)
        .collect();
    arguments.extend(flag.map(OsString::from));
    arguments
}

#[cfg(test)]
mod tests {
    use super::super::installer::Package;
    use super::*;

    fn arguments(list: &[&str]) -> impl Iterator<Item = OsString> {
        list.iter()
            .map(OsString::from)
            .collect::<Vec<_>>()
            .into_iter()
    }

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
}
