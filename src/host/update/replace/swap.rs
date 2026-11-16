use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::log_error;

pub const OLD_SUFFIX: &str = ".old";
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
        log_error!("the previous version stays on disk: {error}");
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
        log_error!("the partial {what} stays on disk: {error}");
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
