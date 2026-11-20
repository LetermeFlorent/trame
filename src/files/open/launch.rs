use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::path::PathBuf;
use std::thread;

use super::is_web;
use crate::log_error;

const OPEN_FAILED: &str = "the link could not be opened";

fn log_open_failure(detail: impl Display) {
    log_error!("{OPEN_FAILED}: {detail}");
}

pub fn open_link(url: String) {
    spawn_open(move || {
        if is_web(&url) {
            launch(OsStr::new(&url));
        }
    });
}

pub(in crate::files) fn launch_path(path: PathBuf) {
    let target: OsString = path.into_os_string();
    spawn_open(move || launch(&target));
}

fn spawn_open(run: impl FnOnce() + Send + 'static) {
    let spawned = thread::Builder::new()
        .name("open-link".to_owned())
        .spawn(run);
    if let Err(error) = spawned {
        log_open_failure(error);
    }
}

#[cfg(windows)]
fn launch(target: &OsStr) {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    use crate::files::picker::com::Apartment;
    use crate::utf16::wide;

    const LOWEST_SUCCESS: isize = 33;
    let verb = wide("open".as_ref());
    let file = wide(target);
    let apartment = Apartment::enter();
    let result = unsafe {
        let null = std::ptr::null();
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            null,
            null,
            SW_SHOWNORMAL,
        )
    };
    drop(apartment);
    if (result as isize) < LOWEST_SUCCESS {
        log_open_failure(format_args!("code {}", result as isize));
    }
}

#[cfg(not(windows))]
fn launch(target: &OsStr) {
    use std::process::{Command, Stdio};

    let program = if cfg!(target_os = "macos") {
        "/usr/bin/open"
    } else {
        "xdg-open"
    };
    let mut command = Command::new(program);
    let started = command
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match started {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(error) => log_open_failure(error),
    }
}
