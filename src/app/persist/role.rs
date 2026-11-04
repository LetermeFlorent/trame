use std::fs::{File, OpenOptions};
use std::io;
use std::mem;
use std::path::{Path, PathBuf};

use trame::files::private_folder;
use trame::log_error;

use super::{FILE_NAME, TRANSCRIPT_DIR};

const LOCK_NAME: &str = "trame.lock";

pub enum Role {
    Owner {
        folder: PathBuf,
        _lock: Option<File>,
    },
    Guest,
}

impl Role {
    pub fn claim(folder: Option<PathBuf>) -> Self {
        let Some(folder) = folder else {
            return Self::Guest;
        };
        match lock(&folder) {
            Ok(lock) => Self::Owner {
                folder,
                _lock: Some(lock),
            },
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => Self::Guest,
            Err(error) => {
                log_error!("the session files could not be locked: {error}");
                Self::Owner {
                    folder,
                    _lock: None,
                }
            }
        }
    }

    pub fn owns(&self) -> bool {
        matches!(self, Self::Owner { .. })
    }

    pub fn session_path(&self) -> Option<PathBuf> {
        self.inside(FILE_NAME)
    }

    pub fn transcript_dir(&self) -> Option<PathBuf> {
        self.inside(TRANSCRIPT_DIR)
    }

    pub fn hand_over(&mut self, start: impl FnOnce() -> io::Result<()>) -> io::Result<()> {
        let folder = match mem::replace(self, Self::Guest) {
            Self::Owner { folder, .. } => Some(folder),
            Self::Guest => None,
        };
        let started = start();
        if started.is_err() {
            *self = Self::claim(folder);
        }
        started
    }

    fn inside(&self, name: &str) -> Option<PathBuf> {
        match self {
            Self::Owner { folder, .. } => Some(folder.join(name)),
            Self::Guest => None,
        }
    }
}

fn lock(folder: &Path) -> io::Result<File> {
    private_folder(folder)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(folder.join(LOCK_NAME))?;
    file.try_lock()?;
    Ok(file)
}
