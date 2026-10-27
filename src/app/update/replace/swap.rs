use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const OLD_SUFFIX: &str = ".old";
const LEFTOVER_ATTEMPTS: u32 = 20;
const LEFTOVER_PAUSE: Duration = Duration::from_millis(500);

pub fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

pub fn remove_leftover(target: &Path) {
    let old = sibling(target, OLD_SUFFIX);
    if !old.exists() {
        return;
    }
    let spawned = thread::Builder::new()
        .name("trame-cleanup".into())
        .spawn(move || {
            for _ in 0..LEFTOVER_ATTEMPTS {
                if remove(&old).is_ok() || !old.exists() {
                    return;
                }
                thread::sleep(LEFTOVER_PAUSE);
            }
        });
    if let Err(error) = spawned {
        eprintln!("trame: the previous version stays on disk: {error}");
    }
}

pub fn remove(path: &Path) -> io::Result<()> {
    match path.is_dir() {
        true => fs::remove_dir_all(path),
        false => fs::remove_file(path),
    }
}

pub fn discard_partial(path: &Path, what: &str) {
    if path.exists()
        && let Err(error) = remove(path)
    {
        eprintln!("trame: the partial {what} stays on disk: {error}");
    }
}

pub fn replace(target: &Path, fresh: &Path) -> io::Result<()> {
    let old = sibling(target, OLD_SUFFIX);
    match remove(&old) {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    fs::rename(target, &old)?;
    let Err(error) = fs::rename(fresh, target) else {
        return Ok(());
    };
    match fs::rename(&old, target) {
        Ok(()) => Err(error),
        Err(back) => Err(io::Error::other(format!(
            "{error}, and the running version could not be put back: {back}"
        ))),
    }
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
    fn a_whole_application_folder_is_swapped_the_same_way() {
        let scratch = Scratch::new("bundle");
        let app = scratch.0.join("trame.app");
        let fresh = scratch.0.join("trame.app.9.9.9.ready");
        let older = sibling(&app, OLD_SUFFIX);
        for (folder, text) in [(&app, "running"), (&fresh, "fresh"), (&older, "older")] {
            fs::create_dir_all(folder).expect("an application folder");
            fs::write(folder.join("version"), text).expect("its content");
        }
        replace(&app, &fresh).expect("the swap");
        assert_eq!(fs::read(app.join("version")).unwrap(), b"fresh");
        assert_eq!(fs::read(older.join("version")).unwrap(), b"running");
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
}
