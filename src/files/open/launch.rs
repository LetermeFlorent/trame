use std::ffi::OsString;
use std::path::PathBuf;
use std::thread;

use super::{LinkTarget, Opening, plan_opening};
use crate::files::home_dir;
use crate::log_error;

pub fn open_link(target: LinkTarget, cwd: Option<PathBuf>) {
    let spawned = thread::Builder::new()
        .name("open-link".to_owned())
        .spawn(move || {
            if let Some(opening) = plan_opening(&target, cwd.as_deref(), home_dir().as_deref()) {
                launch(&opening);
            }
        });
    if let Err(error) = spawned {
        log_error!("the link could not be opened: {error}");
    }
}

fn argument(opening: &Opening) -> OsString {
    match opening {
        Opening::Browser(url) => OsString::from(url),
        Opening::File(path) | Opening::Folder(path) => path.clone().into_os_string(),
    }
}

#[cfg(windows)]
fn launch(opening: &Opening) {
    use windows_sys::Win32::System::Com::{
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
    };
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    use crate::utf16::wide;

    const LOWEST_SUCCESS: isize = 33;
    let verb = wide("open".as_ref());
    let file = wide(&argument(opening));
    let mode = (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32;
    let started = unsafe { CoInitializeEx(std::ptr::null(), mode) };
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
    if started >= 0 {
        unsafe { CoUninitialize() };
    }
    if (result as isize) < LOWEST_SUCCESS {
        log_error!("the link could not be opened: code {}", result as isize);
    }
}

#[cfg(not(windows))]
fn launch(opening: &Opening) {
    use std::process::{Command, Stdio};

    let program = if cfg!(target_os = "macos") {
        "/usr/bin/open"
    } else {
        "xdg-open"
    };
    let mut command = Command::new(program);
    if cfg!(target_os = "macos") && reveals(opening) {
        command.arg("-R");
    }
    let started = command
        .arg(argument(opening))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match started {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(error) => log_error!("the link could not be opened: {error}"),
    }
}

#[cfg(not(windows))]
fn reveals(opening: &Opening) -> bool {
    matches!(opening, Opening::Folder(path) if path.extension().is_some())
}
