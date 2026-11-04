use windows_sys::Win32::Graphics::Dwm::{
    DWMSBT_AUTO, DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE, DwmExtendFrameIntoClientArea,
};
use windows_sys::Win32::UI::Controls::MARGINS;
use winit::window::Window;

use crate::app::window::{dwm, hwnd};

pub fn backdrop_supported(window: &Window) -> bool {
    hwnd(window)
        .is_some_and(|hwnd| dwm::set_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, DWMSBT_AUTO))
}

pub fn set_backdrop(window: &Window, on: bool) {
    let Some(hwnd) = hwnd(window) else {
        return;
    };
    let (kind, edge) = match on {
        true => (DWMSBT_TRANSIENTWINDOW, -1),
        false => (DWMSBT_AUTO, 0),
    };
    let margins = MARGINS {
        cxLeftWidth: edge,
        cxRightWidth: edge,
        cyTopHeight: edge,
        cyBottomHeight: edge,
    };
    unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins) };
    dwm::set_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, kind);
}
