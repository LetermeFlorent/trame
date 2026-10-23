#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::path::PathBuf;
use std::thread;

#[cfg(unix)]
use unix::choose;
#[cfg(windows)]
use windows::choose;

pub struct FolderRequest {
    pub owner: Option<isize>,
    pub title: String,
    pub start: Option<PathBuf>,
}

pub fn pick_folder(request: FolderRequest, reply: impl FnOnce(PathBuf) + Send + 'static) {
    let spawned = thread::Builder::new()
        .name("trame-folder".into())
        .spawn(move || {
            if let Some(folder) = choose(&request) {
                reply(folder);
            }
        });
    if let Err(error) = spawned {
        eprintln!("trame: the folder window could not open: {error}");
    }
}
