mod fades;
mod layout;
mod popup;
mod scroll;
mod tween;

use std::time::Instant;

use trame::text::TextSystem;
use trame::ui::{FrameLayout, Motion};
use trame::workspace::Workspace;

use crate::frame_with;

pub fn settle(text: &mut TextSystem, workspace: &Workspace) -> (Motion, Instant) {
    let start = Instant::now();
    let mut motion = Motion::new(start);
    layout_at(text, workspace, &mut motion, start);
    (motion, start)
}

pub fn layout_at(
    text: &mut TextSystem,
    workspace: &Workspace,
    motion: &mut Motion,
    now: Instant,
) -> FrameLayout {
    motion.tick(now);
    let frame = frame_with(text, workspace, motion);
    motion.finish_layout();
    frame
}
