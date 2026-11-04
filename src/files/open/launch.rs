use std::thread;

use super::is_web;
use crate::log_error;

const OPEN_FAILED: &str = "the link could not be opened";

pub fn open_link(url: String) {
    let spawned = thread::Builder::new()
        .name("open-link".to_owned())
        .spawn(move || {
            if is_web(&url) {
                launch(&url);
            }
        });
    if let Err(error) = spawned {
        log_error!("{OPEN_FAILED}: {error}");
    }
}

#[cfg(windows)]
fn launch(url: &str) {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    use crate::files::picker::com::Apartment;
    use crate::utf16::wide;

    const LOWEST_SUCCESS: isize = 33;
    let verb = wide("open".as_ref());
    let file = wide(url.as_ref());
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
        log_error!("{OPEN_FAILED}: code {}", result as isize);
    }
}

#[cfg(not(windows))]
fn launch(url: &str) {
    use std::process::{Command, Stdio};

    let program = if cfg!(target_os = "macos") {
        "/usr/bin/open"
    } else {
        "xdg-open"
    };
    let mut command = Command::new(program);
    let started = command
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match started {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(error) => log_error!("{OPEN_FAILED}: {error}"),
    }
}
