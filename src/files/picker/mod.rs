#[cfg(windows)]
mod com;
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

use crate::log_error;

pub struct PickRequest {
    pub owner: Option<isize>,
    pub title: String,
    pub start: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PickKind {
    Folder,
    Image,
}

pub fn pick_folder(request: PickRequest, reply: impl FnOnce(PathBuf) + Send + 'static) {
    spawn(request, PickKind::Folder, reply);
}

pub fn pick_image(request: PickRequest, reply: impl FnOnce(PathBuf) + Send + 'static) {
    spawn(request, PickKind::Image, reply);
}

fn spawn(request: PickRequest, kind: PickKind, reply: impl FnOnce(PathBuf) + Send + 'static) {
    let spawned = thread::Builder::new()
        .name("trame-picker".into())
        .spawn(move || {
            if let Some(path) = choose(&request, kind) {
                reply(path);
            }
        });
    if let Err(error) = spawned {
        log_error!("the file window could not open: {error}");
    }
}
