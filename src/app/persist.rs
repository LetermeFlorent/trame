use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use trame::workspace::{Snapshot, Workspace};

const APP_DIR: &str = "trame";
const FILE_NAME: &str = "session.json";
const TEMPORARY_EXTENSION: &str = "json.tmp";
const SAVE_INTERVAL: Duration = Duration::from_secs(1);

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

pub fn session_path() -> Option<PathBuf> {
    state_dir().map(|dir| dir.join(APP_DIR).join(FILE_NAME))
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
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let temporary = path.with_extension(TEMPORARY_EXTENSION);
    let mut file = File::create(&temporary)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, path).inspect_err(|_| {
        let _ = fs::remove_file(&temporary);
    })
}

pub struct Saver {
    path: Option<PathBuf>,
    last_layout: Option<Workspace>,
    last_write: Option<Instant>,
    dirty: bool,
}

impl Saver {
    pub fn new() -> Self {
        Self {
            path: session_path(),
            last_layout: None,
            last_write: None,
            dirty: false,
        }
    }

    pub fn baseline(&mut self, workspace: &Workspace) {
        self.last_layout = Some(workspace.clone());
    }

    pub fn watch(&mut self, workspace: &Workspace) {
        if self.last_layout.as_ref() != Some(workspace) {
            self.last_layout = Some(workspace.clone());
            self.dirty = true;
        }
    }

    pub fn due(&self) -> Option<Instant> {
        if !self.dirty {
            return None;
        }
        Some(
            self.last_write
                .map_or_else(Instant::now, |last| last + SAVE_INTERVAL),
        )
    }

    pub fn save(&mut self, snapshot: &Snapshot) {
        self.dirty = false;
        self.last_write = Some(Instant::now());
        let Some(path) = &self.path else {
            return;
        };
        if let Err(error) = write(path, snapshot) {
            eprintln!("trame: the session could not be saved: {error}");
        }
    }
}
