use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use super::installer::Package;
use super::replace::{bundle, swap};

const PROBE_SUFFIX: &str = ".probe";
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

    fn pick(writable: bool, installed: bool) -> Option<Self> {
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

fn writable(path: &Path) -> bool {
    let probe = swap::sibling(path, PROBE_SUFFIX);
    File::create(&probe)
        .and_then(|_| fs::remove_file(&probe))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
