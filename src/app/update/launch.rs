use std::env;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::Command;

use super::route::Route;
use super::{INSTALL_FLAG, UPDATED_FLAG, installer};

pub fn relaunch(exe: &Path) -> io::Result<()> {
    spawn_again(exe, Some(UPDATED_FLAG))
}

pub fn restart(offer: &str) -> io::Result<()> {
    let exe = env::current_exe()?;
    match Route::of(&exe) {
        Some(route) if !route.replaces() => installer::run(route, offer, &exe),
        _ => spawn_again(&exe, Some(INSTALL_FLAG)),
    }
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
    use super::*;

    fn arguments(list: &[&str]) -> impl Iterator<Item = OsString> {
        list.iter()
            .map(OsString::from)
            .collect::<Vec<_>>()
            .into_iter()
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
    fn starting_again_drops_both_flags() {
        let next = next_arguments(arguments(&[INSTALL_FLAG, "C:/work", UPDATED_FLAG]), None);
        assert_eq!(next, [OsString::from("C:/work")]);
    }
}
