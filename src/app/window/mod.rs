use winit::dpi::LogicalSize;
use winit::window::{CursorIcon, ResizeDirection, Theme, Window, WindowAttributes};

use trame::theme::{Appearance, system};
use trame::ui::ResizeEdge;
use trame::ui::chrome::logo;
use trame::workspace::WindowSnapshot;

#[cfg(windows)]
pub mod dwm;
mod frame;

pub use frame::{frame_border, round_corners};

const APP_NAME: &str = "trame";
const INITIAL_SIZE: LogicalSize<f64> = LogicalSize::new(1180.0, 740.0);
const MINIMUM_SIZE: LogicalSize<f64> = LogicalSize::new(520.0, 320.0);
const MAXIMUM_SAVED_EXTENT: f64 = 16384.0;

pub fn attributes() -> WindowAttributes {
    let attributes = Window::default_attributes()
        .with_title(APP_NAME)
        .with_window_icon(logo::window_icon())
        .with_visible(false)
        .with_inner_size(INITIAL_SIZE)
        .with_min_inner_size(MINIMUM_SIZE);
    platform_attributes(attributes)
}

pub fn restored_attributes(saved: Option<WindowSnapshot>) -> WindowAttributes {
    let attributes = attributes();
    let Some(saved) = saved else {
        return attributes;
    };
    let width = f64::from(saved.width).clamp(MINIMUM_SIZE.width, MAXIMUM_SAVED_EXTENT);
    let height = f64::from(saved.height).clamp(MINIMUM_SIZE.height, MAXIMUM_SAVED_EXTENT);
    attributes
        .with_inner_size(LogicalSize::new(width, height))
        .with_maximized(saved.maximized)
}

pub fn snapshot(window: &Window) -> Option<WindowSnapshot> {
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    if window.is_minimized() == Some(true) || size.width < 1.0 || size.height < 1.0 {
        return None;
    }
    Some(WindowSnapshot {
        width: size.width.round() as u32,
        height: size.height.round() as u32,
        maximized: window.is_maximized(),
    })
}

#[cfg(windows)]
fn platform_attributes(attributes: WindowAttributes) -> WindowAttributes {
    use winit::platform::windows::WindowAttributesExtWindows;
    attributes
        .with_decorations(false)
        .with_undecorated_shadow(true)
}

#[cfg(target_os = "macos")]
fn platform_attributes(attributes: WindowAttributes) -> WindowAttributes {
    use winit::platform::macos::WindowAttributesExtMacOS;
    attributes
        .with_titlebar_transparent(true)
        .with_title_hidden(true)
        .with_fullsize_content_view(true)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_attributes(attributes: WindowAttributes) -> WindowAttributes {
    use winit::platform::wayland::WindowAttributesExtWayland;
    use winit::platform::x11::WindowAttributesExtX11;
    let attributes = WindowAttributesExtWayland::with_name(attributes, APP_NAME, APP_NAME);
    WindowAttributesExtX11::with_name(attributes, APP_NAME, APP_NAME).with_decorations(false)
}

pub fn has_area(window: &Window) -> bool {
    let size = window.inner_size();
    window.is_minimized() != Some(true) && size.width > 0 && size.height > 0
}

pub fn owner(window: &Window) -> Option<isize> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
        _ => None,
    }
}

#[cfg(windows)]
pub fn hwnd(window: &Window) -> Option<windows_sys::Win32::Foundation::HWND> {
    owner(window).map(|handle| handle as _)
}

pub fn appearance(window: &Window) -> Appearance {
    window
        .theme()
        .map(appearance_of)
        .or_else(system::detect)
        .unwrap_or_default()
}

pub fn appearance_of(theme: Theme) -> Appearance {
    match theme {
        Theme::Light => Appearance::Light,
        Theme::Dark => Appearance::Dark,
    }
}

pub fn resize_direction(edge: ResizeEdge) -> ResizeDirection {
    match edge {
        ResizeEdge::North => ResizeDirection::North,
        ResizeEdge::South => ResizeDirection::South,
        ResizeEdge::East => ResizeDirection::East,
        ResizeEdge::West => ResizeDirection::West,
        ResizeEdge::NorthEast => ResizeDirection::NorthEast,
        ResizeEdge::NorthWest => ResizeDirection::NorthWest,
        ResizeEdge::SouthEast => ResizeDirection::SouthEast,
        ResizeEdge::SouthWest => ResizeDirection::SouthWest,
    }
}

pub fn resize_cursor(edge: ResizeEdge) -> CursorIcon {
    match edge {
        ResizeEdge::North | ResizeEdge::South => CursorIcon::NsResize,
        ResizeEdge::East | ResizeEdge::West => CursorIcon::EwResize,
        ResizeEdge::NorthEast | ResizeEdge::SouthWest => CursorIcon::NeswResize,
        ResizeEdge::NorthWest | ResizeEdge::SouthEast => CursorIcon::NwseResize,
    }
}
