use std::fs::{self, File};
use std::io;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};

#[cfg(unix)]
const OWNER_ONLY_FOLDER: u32 = 0o700;
#[cfg(unix)]
const OWNER_ONLY_FILE: u32 = 0o600;

pub fn private_folder(folder: &Path) -> io::Result<()> {
    create_folder(folder)?;
    narrow(folder)
}

#[cfg(unix)]
pub(crate) fn create_folder(folder: &Path) -> io::Result<()> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(OWNER_ONLY_FOLDER)
        .create(folder)
}

#[cfg(unix)]
pub(crate) fn create_file(path: &Path) -> io::Result<File> {
    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(OWNER_ONLY_FILE)
        .open(path)
}

#[cfg(unix)]
pub(super) fn keep_mode(path: &Path, file: &File) -> io::Result<()> {
    match fs::metadata(path) {
        Ok(old) => file.set_permissions(old.permissions()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn narrow(folder: &Path) -> io::Result<()> {
    fs::set_permissions(folder, fs::Permissions::from_mode(OWNER_ONLY_FOLDER))
}

#[cfg(not(unix))]
pub(crate) fn create_folder(folder: &Path) -> io::Result<()> {
    fs::create_dir_all(folder)
}

#[cfg(not(unix))]
pub(crate) fn create_file(path: &Path) -> io::Result<File> {
    File::create(path)
}

#[cfg(not(unix))]
pub(super) fn keep_mode(_path: &Path, _file: &File) -> io::Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn narrow(_folder: &Path) -> io::Result<()> {
    Ok(())
}
