mod saver;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use trame::files::write_atomic;
use trame::workspace::Snapshot;

pub use saver::Saver;

const APP_DIR: &str = "trame";
const FILE_NAME: &str = "session.json";
const TRANSCRIPT_DIR: &str = "text";

pub fn home_dir() -> Option<PathBuf> {
    let key = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    env::var_os(key)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

#[cfg(windows)]
fn state_dir() -> Option<PathBuf> {
    env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn state_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join("Library").join("Application Support"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn state_dir() -> Option<PathBuf> {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| home_dir().map(|home| home.join(".local").join("state")))
}

fn app_dir() -> Option<PathBuf> {
    state_dir().map(|dir| dir.join(APP_DIR))
}

pub fn session_path() -> Option<PathBuf> {
    app_dir().map(|dir| dir.join(FILE_NAME))
}

pub fn transcript_dir() -> Option<PathBuf> {
    app_dir().map(|dir| dir.join(TRANSCRIPT_DIR))
}

pub fn load() -> Option<Snapshot> {
    let path = session_path()?;
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
        Err(error) => {
            eprintln!("trame: the session file could not be read: {error}");
            return None;
        }
    };
    match Snapshot::from_json(&text) {
        Ok(snapshot) => Some(snapshot),
        Err(error) => {
            eprintln!("trame: {error}, starting a fresh session");
            None
        }
    }
}

pub fn write(path: &Path, snapshot: &Snapshot) -> io::Result<()> {
    let text = snapshot.to_json().map_err(io::Error::other)?;
    write_atomic(path, text.as_bytes())
}
