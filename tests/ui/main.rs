mod drop;
mod editor;
mod motion;

use trame::session::SessionId;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::ui::{FrameInput, FrameLayout, UiMetrics};
use trame::workspace::Workspace;

pub const WIDTH: f32 = 1200.0;
pub const HEIGHT: f32 = 700.0;
const FONT_SIZE: f32 = 14.0;

pub fn workspace() -> Workspace {
    let mut workspace = Workspace::new();
    workspace.add_instance(None, SessionId(1));
    workspace.add_tab(SessionId(2));
    workspace.add_instance(None, SessionId(3));
    workspace.active = 0;
    workspace
}

pub fn text_system() -> TextSystem {
    TextSystem::new(FONT_SIZE, DEFAULT_LINE_HEIGHT).expect("a monospace font is installed")
}

pub fn frame(text: &mut TextSystem, workspace: &Workspace) -> FrameLayout {
    let title = |id: SessionId| format!("tab {}", id.0);
    let input = FrameInput {
        workspace,
        width: WIDTH,
        height: HEIGHT,
        metrics: UiMetrics::new(1.0),
        maximized: false,
        title: &title,
    };
    FrameLayout::compute(text, &input)
}
