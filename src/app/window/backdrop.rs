use winit::window::Window;

use trame::theme::Appearance;

#[cfg(windows)]
pub fn set_backdrop(window: &Window, acrylic: bool, appearance: Appearance) -> bool {
    use windows_sys::Win32::Graphics::Dwm::{
        DWMSBT_AUTO, DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
        DWMWA_USE_IMMERSIVE_DARK_MODE, DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
    };
    use windows_sys::Win32::UI::Controls::MARGINS;
    let Some(hwnd) = super::hwnd(window) else {
        return false;
    };
    let dark = i32::from(appearance == Appearance::Dark);
    let kind = match acrylic {
        true => DWMSBT_TRANSIENTWINDOW,
        false => DWMSBT_AUTO,
    };
    let extent = match acrylic {
        true => -1,
        false => 0,
    };
    let margins = MARGINS {
        cxLeftWidth: extent,
        cxRightWidth: extent,
        cyTopHeight: extent,
        cyBottomHeight: extent,
    };
    let set = |attribute, value: &i32| unsafe {
        DwmSetWindowAttribute(
            hwnd,
            attribute as u32,
            (&raw const *value).cast(),
            size_of_val(value) as u32,
        )
    };
    if acrylic {
        set(DWMWA_USE_IMMERSIVE_DARK_MODE, &dark);
    }
    let extended = unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins) };
    let backdrop = set(DWMWA_SYSTEMBACKDROP_TYPE, &kind);
    acrylic && extended == 0 && backdrop == 0
}

#[cfg(not(windows))]
pub fn set_backdrop(_window: &Window, _acrylic: bool, _appearance: Appearance) -> bool {
    false
}
