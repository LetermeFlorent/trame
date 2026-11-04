use winit::window::Window;

#[cfg(windows)]
pub fn backdrop_supported(window: &Window) -> bool {
    use windows_sys::Win32::Graphics::Dwm::{DWMSBT_AUTO, DWMWA_SYSTEMBACKDROP_TYPE};
    super::hwnd(window)
        .is_some_and(|hwnd| super::dwm::set_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, DWMSBT_AUTO))
}

#[cfg(windows)]
pub fn set_backdrop(window: &Window, on: bool) {
    use windows_sys::Win32::Graphics::Dwm::{
        DWMSBT_AUTO, DWMSBT_MAINWINDOW, DWMWA_SYSTEMBACKDROP_TYPE, DwmExtendFrameIntoClientArea,
    };
    use windows_sys::Win32::UI::Controls::MARGINS;
    let Some(hwnd) = super::hwnd(window) else {
        return;
    };
    let (kind, edge) = match on {
        true => (DWMSBT_MAINWINDOW, -1),
        false => (DWMSBT_AUTO, 0),
    };
    let margins = MARGINS {
        cxLeftWidth: edge,
        cxRightWidth: edge,
        cyTopHeight: edge,
        cyBottomHeight: edge,
    };
    unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins) };
    super::dwm::set_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, kind);
}

#[cfg(not(windows))]
pub fn backdrop_supported(_window: &Window) -> bool {
    false
}

#[cfg(not(windows))]
pub fn set_backdrop(_window: &Window, _on: bool) {}
