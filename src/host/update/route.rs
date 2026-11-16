use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use super::installer::Package;
use super::replace::{bundle, swap};

pub const PROBE_SUFFIX: &str = ".probe";
const UNINSTALLER: &str = "unins000.exe";
const PROGRAM: &str = "trame.exe";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Swap,
    Bundle,
    Setup,
    Package(Package),
}

impl Route {
    pub fn of(exe: &Path) -> Option<Self> {
        match env::consts::OS {
            "windows" => Self::on_windows(exe),
            "macos" => bundle::root(exe)
                .filter(|app| writable(app))
                .map(|_| Self::Bundle),
            "linux" => Package::of(exe).map(Self::Package),
            _ => None,
        }
    }

    fn on_windows(exe: &Path) -> Option<Self> {
        let installed = exe
            .parent()
            .is_some_and(|folder| folder.join(UNINSTALLER).exists());
        Self::pick(writable(exe), installed)
    }

    pub fn pick(writable: bool, installed: bool) -> Option<Self> {
        match (writable, installed) {
            (true, _) => Some(Self::Swap),
            (false, true) => Some(Self::Setup),
            (false, false) => None,
        }
    }

    pub fn replaces(self) -> bool {
        matches!(self, Self::Swap | Self::Bundle)
    }

    pub fn target(self, exe: &Path) -> PathBuf {
        let bundle = match self {
            Self::Bundle => bundle::root(exe),
            _ => None,
        };
        bundle.unwrap_or_else(|| exe.to_path_buf())
    }

    pub fn asset(self, number: &str) -> String {
        match self {
            Self::Swap => PROGRAM.to_owned(),
            Self::Bundle => format!("trame-{number}-macos.zip"),
            Self::Setup => format!("trame-{number}-setup.exe"),
            Self::Package(package) => package.asset(number),
        }
    }
}

pub fn writable(path: &Path) -> bool {
    let probe = swap::sibling(path, PROBE_SUFFIX);
    File::create(&probe)
        .and_then(|_| fs::remove_file(&probe))
        .is_ok()
}
