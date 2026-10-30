use winit::window::Window;

use trame::geometry::Point;

#[cfg(windows)]
pub(super) fn cursor_in(window: &Window) -> Option<Point> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let hwnd = crate::app::window::hwnd(window)?;
    let mut point = POINT { x: 0, y: 0 };
    let read =
        unsafe { GetCursorPos(&raw mut point) != 0 && ScreenToClient(hwnd, &raw mut point) != 0 };
    read.then(|| Point::new(point.x as f32, point.y as f32))
}

#[cfg(not(windows))]
pub(super) fn cursor_in(_window: &Window) -> Option<Point> {
    None
}
