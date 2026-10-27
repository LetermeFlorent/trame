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
    use windows_sys::Win32::Graphics::Dwm::DWMWA_VISIBLE_FRAME_BORDER_THICKNESS;
    let Some(hwnd) = super::hwnd(window) else {
        return 0.0;
    };
    super::dwm::attribute(hwnd, DWMWA_VISIBLE_FRAME_BORDER_THICKNESS, 0u32)
        .map_or(0.0, |thickness| thickness as f32)
}

#[cfg(not(windows))]
pub fn round_corners(_window: &Window) {}

#[cfg(not(windows))]
pub fn frame_border(_window: &Window) -> f32 {
    0.0
}
