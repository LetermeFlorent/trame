mod icons;
mod picker;
mod places;

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub use icons::{ICON_SIZE, IconStore, fit, import_icon, load_icon};
pub use picker::{PickRequest, pick_folder, pick_image};
pub use places::{home_dir, short_path};

const TEMPORARY_SUFFIX: &str = ".tmp";

pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_owned());
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(TEMPORARY_SUFFIX);
    let temporary = PathBuf::from(temporary);
    fill(&temporary, &path, bytes)
        .and_then(|()| fs::rename(&temporary, &path))
        .inspect_err(|_| {
            let _ = fs::remove_file(&temporary);
        })
}

fn fill(temporary: &Path, path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(temporary)?;
    keep_mode(path, &file)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(unix)]
fn keep_mode(path: &Path, file: &File) -> io::Result<()> {
    match fs::metadata(path) {
        Ok(old) => file.set_permissions(old.permissions()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(not(unix))]
fn keep_mode(_path: &Path, _file: &File) -> io::Result<()> {
    Ok(())
}
