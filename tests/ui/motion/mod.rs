mod fades;
mod layout;
mod scroll;
mod tween;

use std::time::Instant;

use trame::session::SessionId;
use trame::text::TextSystem;
use trame::ui::{FrameInput, FrameLayout, Motion, UiMetrics};
use trame::workspace::Workspace;

use crate::{HEIGHT, WIDTH};

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
    let title = |id: SessionId| format!("tab {}", id.0);
    let input = FrameInput {
        workspace,
        width: WIDTH,
        height: HEIGHT,
        metrics: UiMetrics::new(1.0),
        maximized: false,
        title: &title,
    };
    motion.tick(now);
    let frame = FrameLayout::compute_with(text, &input, motion);
    motion.finish_layout();
    frame
}
