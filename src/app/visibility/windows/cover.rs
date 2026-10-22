use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    COMPLEXREGION, CombineRgn, CreateRectRgn, CreateRectRgnIndirect, DeleteObject, GetWindowRgn,
    NULLREGION, RGN_DIFF,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GWL_EXSTYLE, GetLayeredWindowAttributes, GetSystemMetrics, GetWindowLongW,
    IsIconic, IsWindowVisible, LWA_ALPHA, LWA_COLORKEY, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
};
use windows_sys::core::BOOL;

use super::{frame_bounds, has_area, is_cloaked};

const OPAQUE: u8 = u8::MAX;

pub(super) fn is_covered(hwnd: HWND) -> bool {
    let bounds = frame_bounds(hwnd).and_then(on_screen);
    let (Some(bounds), Some(above)) = (bounds, windows_above(hwnd)) else {
        return false;
    };
    unsafe {
        let uncovered = CreateRectRgnIndirect(&raw const bounds);
        let covered = above.into_iter().filter_map(opaque_bounds).any(|rect| {
            let piece = CreateRectRgnIndirect(&raw const rect);
            let left = CombineRgn(uncovered, uncovered, piece, RGN_DIFF);
            DeleteObject(piece);
            left == NULLREGION
        });
        DeleteObject(uncovered);
        covered
    }
}

struct Walk {
    target: HWND,
    above: Vec<HWND>,
    found: bool,
}

fn windows_above(hwnd: HWND) -> Option<Vec<HWND>> {
    let mut walk = Walk {
        target: hwnd,
        above: Vec::new(),
        found: false,
    };
    unsafe { EnumWindows(Some(collect_above), (&raw mut walk) as LPARAM) };
    walk.found.then_some(walk.above)
}

unsafe extern "system" fn collect_above(hwnd: HWND, data: LPARAM) -> BOOL {
    let walk = unsafe { &mut *(data as *mut Walk) };
    if hwnd == walk.target {
        walk.found = true;
        return 0;
    }
    walk.above.push(hwnd);
    1
}

fn opaque_bounds(hwnd: HWND) -> Option<RECT> {
    let shown = unsafe { IsWindowVisible(hwnd) != 0 && IsIconic(hwnd) == 0 };
    if !shown || is_see_through(hwnd) || has_shaped_region(hwnd) || is_cloaked(hwnd) {
        return None;
    }
    frame_bounds(hwnd)
}

fn is_see_through(hwnd: HWND) -> bool {
    let style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32;
    if style & (WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW) != 0 {
        return true;
    }
    style & WS_EX_LAYERED != 0 && !is_opaque_layer(hwnd)
}

fn is_opaque_layer(hwnd: HWND) -> bool {
    let (mut key, mut alpha, mut flags) = (0, 0, 0);
    let known = unsafe {
        GetLayeredWindowAttributes(hwnd, &raw mut key, &raw mut alpha, &raw mut flags) != 0
    };
    known && flags & LWA_COLORKEY == 0 && (flags & LWA_ALPHA == 0 || alpha == OPAQUE)
}

fn has_shaped_region(hwnd: HWND) -> bool {
    unsafe {
        let region = CreateRectRgn(0, 0, 0, 0);
        let shaped = GetWindowRgn(hwnd, region) == COMPLEXREGION;
        DeleteObject(region);
        shaped
    }
}

fn on_screen(rect: RECT) -> Option<RECT> {
    let (x, y, width, height) = unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    };
    let clipped = RECT {
        left: rect.left.max(x),
        top: rect.top.max(y),
        right: rect.right.min(x + width),
        bottom: rect.bottom.min(y + height),
    };
    has_area(&clipped).then_some(clipped)
}
