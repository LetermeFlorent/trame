use trame::desk::Desk;
use trame::ui::{FrameLayout, Motion, UiMetrics};

use super::super::frame;
use super::surface::Surface;

pub(super) struct FirstFrame {
    pub(super) metrics: UiMetrics,
    pub(super) frame: FrameLayout,
    pub(super) motion: Motion,
}

pub(super) fn first_frame(surface: &mut Surface, desk: &Desk) -> FirstFrame {
    let metrics = surface.metrics(&desk.preferences);
    let mut motion = Motion::default();
    let layout_input = frame::LayoutInput {
        workspace: &desk.workspace,
        window: &surface.window,
        metrics,
        folder: None,
        update: None,
        unsaved: !desk.saver.keeps_session(),
        update_failed: false,
        strings: desk.strings,
    };
    let frame = frame::compute(&mut surface.text, &layout_input, &mut motion, &|_| {
        String::new()
    });
    FirstFrame {
        metrics,
        frame,
        motion,
    }
}
