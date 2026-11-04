use objc2::rc::Retained;
use objc2::runtime::NSObjectProtocol;
use objc2::{MainThreadMarker, class};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSWindowOrderingMode,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

pub fn backdrop_supported(window: &Window) -> bool {
    content_view(window).is_some()
}

pub fn set_backdrop(window: &Window, on: bool) {
    let (Some(view), Some(marker)) = (content_view(window), MainThreadMarker::new()) else {
        return;
    };
    for child in view.subviews() {
        if child.isKindOfClass(class!(NSVisualEffectView)) {
            child.removeFromSuperview();
        }
    }
    if on {
        add_blur(&view, marker);
    }
}

fn add_blur(view: &NSView, marker: MainThreadMarker) {
    let blur = NSVisualEffectView::initWithFrame(marker.alloc(), view.bounds());
    blur.setMaterial(NSVisualEffectMaterial::Sidebar);
    blur.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
    blur.setState(NSVisualEffectState::FollowsWindowActiveState);
    blur.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    view.addSubview_positioned_relativeTo(&blur, NSWindowOrderingMode::Below, None);
    let layers = view.layer().and_then(|layer| unsafe { layer.sublayers() });
    for layer in layers.iter().flat_map(|layers| layers.iter()) {
        if layer.isKindOfClass(class!(CAMetalLayer)) {
            layer.setZPosition(1.0);
        }
    }
}

fn content_view(window: &Window) -> Option<Retained<NSView>> {
    match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::AppKit(handle) => unsafe {
            Retained::retain(handle.ns_view.as_ptr().cast())
        },
        _ => None,
    }
}
