use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use super::UPDATED_FLAG;

const OLD_SUFFIX: &str = ".old";
const SMALLEST_EXECUTABLE: u64 = 1024 * 1024;
const EXECUTABLE_MAGIC: &[u8; 2] = b"MZ";
const LEFTOVER_ATTEMPTS: u32 = 20;
const LEFTOVER_PAUSE: Duration = Duration::from_millis(500);

pub fn sibling(exe: &Path, suffix: &str) -> PathBuf {
    let mut name = exe.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

pub fn remove_leftover(exe: &Path) {
    let old = sibling(exe, OLD_SUFFIX);
    if !old.exists() {
        return;
    }
    let spawned = thread::Builder::new()
        .name("trame-cleanup".into())
        .spawn(move || {
            for _ in 0..LEFTOVER_ATTEMPTS {
                if fs::remove_file(&old).is_ok() || !old.exists() {
                    return;
                }
                thread::sleep(LEFTOVER_PAUSE);
            }
        });
    if let Err(error) = spawned {
        eprintln!("trame: the previous version stays on disk: {error}");
    }
}

pub fn check_executable(path: &Path) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut magic = [0; 2];
    file.read_exact(&mut magic)?;
    let large_enough = file.metadata()?.len() >= SMALLEST_EXECUTABLE;
    match large_enough && magic == *EXECUTABLE_MAGIC {
        true => Ok(()),
        false => Err(io::Error::other("the download is not a program")),
    }
}

pub fn replace(exe: &Path, fresh: &Path) -> io::Result<()> {
    let old = sibling(exe, OLD_SUFFIX);
    match fs::remove_file(&old) {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    fs::rename(exe, &old)?;
    let Err(error) = fs::rename(fresh, exe) else {
        return Ok(());
    };
    match fs::rename(&old, exe) {
        Ok(()) => Err(error),
        Err(back) => Err(io::Error::other(format!(
            "{error}, and the running version could not be put back: {back}"
        ))),
    }
}

pub fn relaunch(exe: &Path) -> io::Result<()> {
    Command::new(exe)
        .args(
            std::env::args_os()
                .skip(1)
                .filter(|arg| arg != UPDATED_FLAG),
        )
        .arg(OsStr::new(UPDATED_FLAG))
        .spawn()
        .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("trame-swap-{name}-{}", std::process::id()));
            fs::create_dir_all(&dir).expect("scratch folder");
            Self(dir)
        }

        fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, bytes).expect("scratch file");
            path
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn program() -> Vec<u8> {
        let mut bytes = EXECUTABLE_MAGIC.to_vec();
        bytes.resize(SMALLEST_EXECUTABLE as usize, 0);
        bytes
    }

    #[test]
    fn the_new_version_takes_the_place_and_the_old_one_is_kept_aside() {
        let scratch = Scratch::new("replace");
        let exe = scratch.file("trame.exe", b"running");
        let fresh = scratch.file("trame.exe.new", b"fresh");
        scratch.file("trame.exe.old", b"older");
        replace(&exe, &fresh).expect("the swap");
        assert_eq!(fs::read(&exe).unwrap(), b"fresh");
        assert_eq!(fs::read(sibling(&exe, OLD_SUFFIX)).unwrap(), b"running");
        assert!(!fresh.exists());
    }

    #[test]
    fn a_failed_swap_puts_the_running_version_back() {
        let scratch = Scratch::new("rollback");
        let exe = scratch.file("trame.exe", b"running");
        let missing = scratch.0.join("trame.exe.new");
        assert!(replace(&exe, &missing).is_err());
        assert_eq!(fs::read(&exe).unwrap(), b"running");
    }

    #[test]
    fn only_a_large_enough_program_is_accepted() {
        let scratch = Scratch::new("check");
        assert!(check_executable(&scratch.file("good", &program())).is_ok());
        assert!(check_executable(&scratch.file("small", b"MZ tiny")).is_err());
        let mut page = program();
        page[..2].copy_from_slice(b"<h");
        assert!(check_executable(&scratch.file("page", &page)).is_err());
        assert!(check_executable(&scratch.file("empty", b"")).is_err());
    }
}
