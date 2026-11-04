use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Dwm::DwmGetWindowAttribute;

pub fn attribute<T>(hwnd: HWND, attribute: i32, mut value: T) -> Option<T> {
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            attribute as u32,
            (&raw mut value).cast(),
            size_of_val(&value) as u32,
        )
    };
    (result == 0).then_some(value)
}
