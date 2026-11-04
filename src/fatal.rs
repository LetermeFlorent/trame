#[cfg(windows)]
pub fn report(message: &str) {
    use std::ptr;
    use trame::utf16::wide;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};

    let text = wide(message.as_ref());
    let caption = wide("trame".as_ref());
    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
pub fn report(message: &str) {
    eprintln!("{message}");
}
