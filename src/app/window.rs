use winit::dpi::LogicalSize;
use winit::window::{CursorIcon, ResizeDirection, Theme, Window, WindowAttributes};

use trame::icon;
use trame::theme::{Appearance, system};
use trame::ui::ResizeEdge;
use trame::workspace::WindowSnapshot;

const APP_NAME: &str = "trame";
const INITIAL_SIZE: LogicalSize<f64> = LogicalSize::new(1180.0, 740.0);
const MINIMUM_SIZE: LogicalSize<f64> = LogicalSize::new(520.0, 320.0);
const MAXIMUM_SAVED_EXTENT: f64 = 16384.0;

pub fn attributes() -> WindowAttributes {
    let attributes = Window::default_attributes()
        .with_title(APP_NAME)
        .with_window_icon(icon::window_icon())
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

#[cfg(windows)]
pub fn round_corners(window: &Window) {
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
    };
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let preference = DWMWCP_ROUND;
    unsafe {
        DwmSetWindowAttribute(
            handle.hwnd.get() as _,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            (&raw const preference).cast(),
            size_of_val(&preference) as u32,
        );
    }
}

#[cfg(not(windows))]
pub fn round_corners(_window: &Window) {}

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
