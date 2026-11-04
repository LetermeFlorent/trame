use trame::locale::Strings;
use trame::ui::{FrameLayout, Motion, UiMetrics};
use trame::workspace::Workspace;

use super::super::frame;
use super::surface::Surface;
use crate::app::persist::Saver;

pub(super) struct FirstFrame {
    pub(super) workspace: Workspace,
    pub(super) metrics: UiMetrics,
    pub(super) frame: FrameLayout,
    pub(super) motion: Motion,
    pub(super) strings: &'static Strings,
}

pub(super) fn first_frame(surface: &mut Surface, saver: &Saver) -> FirstFrame {
    let metrics = surface.metrics();
    let workspace = Workspace::numbered(surface.preferences.numbering);
    let strings = surface.preferences.language.resolve().strings();
    let mut motion = Motion::default();
    let layout_input = frame::LayoutInput {
        workspace: &workspace,
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
    FirstFrame {
        workspace,
        metrics,
        frame,
        motion,
        strings,
    }
}
