mod icons;
mod log;
mod modes;
mod picker;
mod places;

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use modes::keep_mode;

pub(crate) use modes::{append_file, create_file, create_folder};

pub use icons::{ICON_SIZE, IconStore, fit, import_icon, load_icon};
pub use log::{LogFile, record_error, start_log};
pub use modes::private_folder;
pub use picker::{PickRequest, pick_folder, pick_image};
pub use places::{home_dir, short_path};

const TEMPORARY_SUFFIX: &str = ".tmp";

pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_owned());
    if let Some(folder) = path.parent() {
        create_folder(folder)?;
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
    let mut file = create_file(temporary)?;
    keep_mode(path, &file)?;
    file.write_all(bytes)?;
    file.sync_all()
}
