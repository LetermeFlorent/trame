mod drop;
mod editor;
mod logo;
mod motion;
mod overlay;
mod sidebar;
mod tabbar;

use trame::locale::Language;
use trame::session::SessionId;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::ui::{FrameInput, FrameLayout, Motion, UiMetrics};
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

fn title(id: SessionId) -> String {
    format!("tab {}", id.0)
}

pub fn frame_with(
    text: &mut TextSystem,
    workspace: &Workspace,
    motion: &mut Motion,
) -> FrameLayout {
    frame_scaled(text, workspace, motion, 1.0)
}

pub fn frame_at(text: &mut TextSystem, workspace: &Workspace, scale: f32) -> FrameLayout {
    frame_scaled(text, workspace, &mut Motion::default(), scale)
}

fn frame_scaled(
    text: &mut TextSystem,
    workspace: &Workspace,
    motion: &mut Motion,
    scale: f32,
) -> FrameLayout {
    let input = FrameInput {
        workspace,
        width: WIDTH,
        height: HEIGHT,
        metrics: UiMetrics::new(scale),
        maximized: false,
        title: &title,
        strings: Language::English.strings(),
    };
    FrameLayout::compute_with(text, &input, motion)
}

pub fn frame(text: &mut TextSystem, workspace: &Workspace) -> FrameLayout {
    frame_with(text, workspace, &mut Motion::default())
}
