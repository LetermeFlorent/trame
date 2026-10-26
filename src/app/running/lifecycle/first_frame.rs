use trame::locale::Strings;
use trame::ui::{FrameLayout, Motion, UiMetrics};
use trame::workspace::Workspace;

use super::super::frame;
use super::surface::Surface;

pub(super) fn first_frame(
    surface: &mut Surface,
    workspace: &Workspace,
    metrics: UiMetrics,
) -> (FrameLayout, Motion, &'static Strings) {
    let strings = surface.preferences.language.resolve().strings();
    let mut motion = Motion::default();
    let layout_input = frame::LayoutInput {
        workspace,
        window: &surface.window,
        metrics,
        folder: None,
        update: None,
        strings,
    };
    let frame = frame::compute(&mut surface.text, &layout_input, &mut motion, &|_| {
        String::new()
    });
    (frame, motion, strings)
}
