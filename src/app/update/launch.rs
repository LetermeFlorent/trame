use std::env;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::Command;

use super::UPDATED_FLAG;

pub fn relaunch(exe: &Path) -> io::Result<()> {
    spawn_again(exe, true)
}

pub fn restart() -> io::Result<()> {
    spawn_again(&env::current_exe()?, false)
}

fn spawn_again(exe: &Path, updated: bool) -> io::Result<()> {
    Command::new(exe)
        .args(next_arguments(env::args_os().skip(1), updated))
        .spawn()
        .map(drop)
}

fn next_arguments(current: impl Iterator<Item = OsString>, updated: bool) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = current.filter(|arg| arg != UPDATED_FLAG).collect();
    if updated {
        arguments.push(UPDATED_FLAG.into());
    }
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
        let next = next_arguments(arguments(&["C:/work", UPDATED_FLAG]), true);
        assert_eq!(next, ["C:/work", UPDATED_FLAG].map(OsString::from));
    }

    #[test]
    fn a_restart_drops_the_flag_so_the_new_start_installs() {
        let next = next_arguments(arguments(&[UPDATED_FLAG, "C:/work"]), false);
        assert_eq!(next, [OsString::from("C:/work")]);
    }
}
