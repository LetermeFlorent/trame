use std::fs::File;
use std::io;
use std::path::Path;
use std::process::Command;

use super::super::replace::signed;

pub const ARGUMENTS: [&str; 5] = [
    "/SILENT",
    "/SUPPRESSMSGBOXES",
    "/NORESTART",
    "/ALLUSERS",
    "/RELAUNCH=yes",
];

pub fn run(installer: &Path, expected: &str) -> io::Result<()> {
    let locked = lock(installer)?;
    signed::confirm(expected, &locked)?;
    let started = Command::new(installer).args(ARGUMENTS).spawn().map(drop);
    drop(locked);
    started
}

#[cfg(windows)]
pub fn lock(path: &Path) -> io::Result<File> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
    OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
}

#[cfg(not(windows))]
pub fn lock(path: &Path) -> io::Result<File> {
    File::open(path)
}
