mod cover;

use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows_sys::Win32::System::StationsAndDesktops::{
    CloseDesktop, DESKTOP_READOBJECTS, GetThreadDesktop, GetUserObjectInformationW, HDESK,
    OpenInputDesktop, UOI_NAME,
};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use winit::window::Window;

use crate::app::window;
use cover::is_covered;

const DESKTOP_NAME_CAPACITY: usize = 256;

type DesktopName = [u16; DESKTOP_NAME_CAPACITY];

pub fn is_hidden(window: &Window) -> bool {
    let Some(hwnd) = window::hwnd(window) else {
        return false;
    };
    off_input_desktop() || is_cloaked(hwnd) || (!window.has_focus() && is_covered(hwnd))
}

fn off_input_desktop() -> bool {
    let input = unsafe { OpenInputDesktop(0, 0, DESKTOP_READOBJECTS) };
    if input.is_null() {
        return false;
    }
    let own = unsafe { GetThreadDesktop(GetCurrentThreadId()) };
    let differs = match (desktop_name(input), desktop_name(own)) {
        (Some(input), Some(own)) => input != own,
        _ => false,
    };
    unsafe { CloseDesktop(input) };
    differs
}

fn desktop_name(desktop: HDESK) -> Option<DesktopName> {
    let mut name: DesktopName = [0; DESKTOP_NAME_CAPACITY];
    let mut needed = 0;
    let read = unsafe {
        GetUserObjectInformationW(
            desktop,
            UOI_NAME,
            name.as_mut_ptr().cast(),
            size_of_val(&name) as u32,
            &raw mut needed,
        )
    };
    (read != 0).then_some(name)
}

fn is_cloaked(hwnd: HWND) -> bool {
    window::dwm::attribute(hwnd, DWMWA_CLOAKED, 0u32).is_some_and(|cloaked| cloaked != 0)
}

fn frame_bounds(hwnd: HWND) -> Option<RECT> {
    let empty = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    window::dwm::attribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, empty).filter(has_area)
}

fn has_area(rect: &RECT) -> bool {
    rect.right > rect.left && rect.bottom > rect.top
}
