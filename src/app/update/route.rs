use std::fs::{self, File};
use std::path::Path;

use super::swap;

const PROBE_SUFFIX: &str = ".probe";
const UNINSTALLER: &str = "unins000.exe";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Swap,
    Setup,
}

impl Route {
    pub fn of(exe: &Path) -> Option<Self> {
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
}

fn writable(exe: &Path) -> bool {
    let probe = swap::sibling(exe, PROBE_SUFFIX);
    File::create(&probe)
        .and_then(|_| fs::remove_file(&probe))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(Route::of(&exe), Some(Route::Swap));
        assert!(!swap::sibling(&exe, PROBE_SUFFIX).exists());
        fs::remove_dir_all(&folder).expect("cleanup");
    }
}
