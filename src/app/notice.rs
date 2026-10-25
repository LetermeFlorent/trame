use std::time::{Duration, Instant};

use winit::window::Window;

const QUIET_FOR: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct Notifier {
    shown_on: Option<isize>,
    last: Option<Instant>,
}

impl Notifier {
    pub fn show(&mut self, window: &Window, title: &str, body: &str) {
        let now = Instant::now();
        if self.last.is_some_and(|last| now < last + QUIET_FOR) {
            return;
        }
        self.last = Some(now);
        if let Some(owner) = owner_of(window) {
            let added = self.shown_on.is_some();
            if tray::show(owner, added, title, body) {
                self.shown_on = Some(owner);
            }
        }
    }

    pub fn clear(&mut self) {
        if let Some(owner) = self.shown_on.take() {
            tray::remove(owner);
        }
    }
}

impl Drop for Notifier {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(windows)]
fn owner_of(window: &Window) -> Option<isize> {
    super::window::hwnd(window).map(|hwnd| hwnd as isize)
}

#[cfg(not(windows))]
fn owner_of(_window: &Window) -> Option<isize> {
    None
}

#[cfg(windows)]
mod tray {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::Shell::{
        NIF_ICON, NIF_INFO, NIF_TIP, NIIF_USER, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
        Shell_NotifyIconW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        ICON_BIG, IDI_APPLICATION, LoadIconW, SendMessageW, WM_GETICON,
    };

    const ICON_ID: u32 = 1;

    pub fn show(owner: isize, added: bool, title: &str, body: &str) -> bool {
        let mut data = data(owner);
        data.uFlags = NIF_ICON | NIF_TIP | NIF_INFO;
        data.hIcon = icon(owner as HWND);
        data.dwInfoFlags = NIIF_USER;
        copy(&mut data.szTip, "trame");
        copy(&mut data.szInfoTitle, title);
        copy(&mut data.szInfo, body);
        let message = if added { NIM_MODIFY } else { NIM_ADD };
        unsafe { Shell_NotifyIconW(message, &data) != 0 }
    }

    pub fn remove(owner: isize) {
        let data = data(owner);
        unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
    }

    fn data(owner: isize) -> NOTIFYICONDATAW {
        let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = owner as HWND;
        data.uID = ICON_ID;
        data
    }

    fn icon(hwnd: HWND) -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
        let own = unsafe { SendMessageW(hwnd, WM_GETICON, ICON_BIG as usize, 0) };
        match own {
            0 => unsafe { LoadIconW(std::ptr::null_mut(), IDI_APPLICATION) },
            handle => handle as _,
        }
    }

    fn copy<const N: usize>(target: &mut [u16; N], text: &str) {
        let units = text.encode_utf16().take(N - 1);
        for (slot, unit) in target.iter_mut().zip(units) {
            *slot = unit;
        }
    }
}

#[cfg(not(windows))]
mod tray {
    pub fn show(_owner: isize, _added: bool, _title: &str, _body: &str) -> bool {
        false
    }

    pub fn remove(_owner: isize) {}
}
