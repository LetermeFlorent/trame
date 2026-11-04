use trame::locale::Strings;
use trame::ui::{FrameLayout, Motion, UiMetrics};
use trame::workspace::Workspace;

use super::super::frame;
use super::surface::Surface;
use crate::app::persist::Saver;

pub(super) fn first_frame(
    surface: &mut Surface,
    workspace: &Workspace,
    metrics: UiMetrics,
    saver: &Saver,
) -> (FrameLayout, Motion, &'static Strings) {
    let strings = surface.preferences.language.resolve().strings();
    let mut motion = Motion::default();
    let layout_input = frame::LayoutInput {
        workspace,
        window: &surface.window,
        metrics,
        folder: None,
        update: None,
        unsaved: !saver.keeps_session(),
        update_failed: false,
        strings,
    };
    let frame = frame::compute(&mut surface.text, &layout_input, &mut motion, &|_| {
        String::new()
    });
    (frame, motion, strings)
}
