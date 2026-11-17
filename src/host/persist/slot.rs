use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::{FILE_NAME, TRANSCRIPT_DIR};

pub const WINDOWS_DIR: &str = "windows";
const FIRST: usize = 0;

#[derive(Clone, Debug)]
pub struct Slot {
    index: usize,
    folder: Option<PathBuf>,
}

impl Slot {
    pub fn new(root: Option<&Path>, index: usize) -> Self {
        Self {
            index,
            folder: root.map(|root| folder_of(root, index)),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn owns(&self) -> bool {
        self.folder.is_some()
    }

    pub fn session_path(&self) -> Option<PathBuf> {
        self.inside(FILE_NAME)
    }

    pub fn transcript_dir(&self) -> Option<PathBuf> {
        self.inside(TRANSCRIPT_DIR)
    }

    pub fn forget(&self) -> io::Result<()> {
        let Some(folder) = &self.folder else {
            return Ok(());
        };
        if self.index != FIRST {
            return missing_is_fine(fs::remove_dir_all(folder));
        }
        missing_is_fine(fs::remove_file(folder.join(FILE_NAME)))?;
        missing_is_fine(fs::remove_dir_all(folder.join(TRANSCRIPT_DIR)))
    }

    fn inside(&self, name: &str) -> Option<PathBuf> {
        self.folder.as_ref().map(|folder| folder.join(name))
    }
}

pub fn saved_slots(root: &Path) -> Vec<usize> {
    let numbered = fs::read_dir(root.join(WINDOWS_DIR))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse::<usize>().ok())
        .filter(|index| *index != FIRST);
    let mut slots: Vec<usize> = std::iter::once(FIRST)
        .chain(numbered)
        .filter(|index| folder_of(root, *index).join(FILE_NAME).is_file())
        .collect();
    slots.sort_unstable();
    slots
}

pub fn free_slot(taken: impl IntoIterator<Item = usize>) -> usize {
    let mut taken: Vec<usize> = taken.into_iter().collect();
    taken.sort_unstable();
    let mut free = FIRST;
    for index in taken {
        if index == free {
            free += 1;
        }
    }
    free
}

fn folder_of(root: &Path, index: usize) -> PathBuf {
    match index {
        FIRST => root.to_owned(),
        _ => root.join(WINDOWS_DIR).join(index.to_string()),
    }
}

fn missing_is_fine(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}
