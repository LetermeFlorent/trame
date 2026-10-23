use winit::window::Window;

#[cfg(windows)]
pub fn round_corners(window: &Window) {
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
    };
    let Some(hwnd) = super::hwnd(window) else {
        return;
    };
    let preference = DWMWCP_ROUND;
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            (&raw const preference).cast(),
            size_of_val(&preference) as u32,
        );
    }
}

#[cfg(windows)]
pub fn frame_border(window: &Window) -> f32 {
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_VISIBLE_FRAME_BORDER_THICKNESS, DwmGetWindowAttribute,
    };
    let Some(hwnd) = super::hwnd(window) else {
        return 0.0;
    };
    let mut thickness = 0u32;
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_VISIBLE_FRAME_BORDER_THICKNESS as u32,
            (&raw mut thickness).cast(),
            size_of_val(&thickness) as u32,
        )
    };
    match result {
        0 => thickness as f32,
        _ => 0.0,
    }
}

#[cfg(not(windows))]
pub fn round_corners(_window: &Window) {}

#[cfg(not(windows))]
pub fn frame_border(_window: &Window) -> f32 {
    0.0
}
