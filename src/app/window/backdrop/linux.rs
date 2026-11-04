use trame::theme::system::kde_desktop;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, PropMode};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

const BLUR_REGION: &[u8] = b"_KDE_NET_WM_BLUR_BEHIND_REGION";

enum Surface {
    Wayland,
    X11(u32),
}

pub fn blur_desktop() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|desktop| kde_desktop(&desktop))
}

pub fn backdrop_supported(window: &Window) -> bool {
    blur_desktop()
        && match surface(window) {
            Some(Surface::Wayland) => true,
            Some(Surface::X11(_)) => blur_atom().is_some(),
            None => false,
        }
}

pub fn set_backdrop(window: &Window, on: bool) {
    match surface(window) {
        Some(Surface::Wayland) => window.set_blur(on),
        Some(Surface::X11(id)) => {
            set_region(id, on);
        }
        None => {}
    }
}

fn surface(window: &Window) -> Option<Surface> {
    match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::Wayland(_) => Some(Surface::Wayland),
        RawWindowHandle::Xlib(handle) => Some(Surface::X11(handle.window as u32)),
        RawWindowHandle::Xcb(handle) => Some(Surface::X11(handle.window.get())),
        _ => None,
    }
}

fn blur_atom() -> Option<(RustConnection, u32)> {
    let (connection, _) = x11rb::connect(None).ok()?;
    let atom = connection
        .intern_atom(true, BLUR_REGION)
        .ok()?
        .reply()
        .ok()?
        .atom;
    (atom != x11rb::NONE).then_some((connection, atom))
}

fn set_region(window: u32, on: bool) -> Option<()> {
    let (connection, atom) = blur_atom()?;
    let request = match on {
        true => {
            connection.change_property32(PropMode::REPLACE, window, atom, AtomEnum::CARDINAL, &[])
        }
        false => connection.delete_property(window, atom),
    };
    request.ok()?.check().ok()
}
