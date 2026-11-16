use std::env;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::route::Route;
use super::version::Version;
use super::{INSTALL_FLAG, UPDATED_FLAG, installer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handoff(pub Next);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Next {
    Relaunch(PathBuf),
    Installer(Route, String, PathBuf),
}

impl Handoff {
    pub fn after(route: Route, exe: &Path, version: Version) -> Self {
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

pub fn next_arguments(
    current: impl Iterator<Item = OsString>,
    flag: Option<&str>,
) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = current
        .filter(|arg| arg != UPDATED_FLAG && arg != INSTALL_FLAG)
        .collect();
    arguments.extend(flag.map(OsString::from));
    arguments
}
