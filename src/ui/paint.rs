use std::collections::HashMap;

use crate::geometry::Point;
use crate::render::Scene;
use crate::session::{Session, SessionId};
use crate::text::TextSystem;
use crate::theme::Theme;
use crate::ui::chrome;
use crate::ui::frame::{FrameLayout, PaneFrame};
use crate::ui::hit::Hit;
use crate::ui::terminal::{TerminalPaint, draw_terminal};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameView {
    pub hover: Option<Hit>,
    pub pressed: Option<Hit>,
    pub window_focused: bool,
}

pub struct FrameSources<'a> {
    pub sessions: &'a HashMap<SessionId, Session>,
}

pub fn draw_frame(
    scene: &mut Scene,
    text: &mut TextSystem,
    theme: &Theme,
    frame: &FrameLayout,
    view: &FrameView,
    sources: &FrameSources,
) {
    chrome::draw_chrome(scene, text, theme, frame, view);
    for pane in &frame.panes {
        draw_body(scene, text, theme, pane, view, sources);
    }
    chrome::draw_dividers(scene, theme, frame, view);
    chrome::draw_editor(scene, text, theme, frame, view);
}

fn draw_body(
    scene: &mut Scene,
    text: &mut TextSystem,
    theme: &Theme,
    pane: &PaneFrame,
    view: &FrameView,
    sources: &FrameSources,
) {
    scene.fill(pane.body, theme.terminal, 0.0);
    let Some(session) = pane.active.and_then(|id| sources.sessions.get(&id)) else {
        return;
    };
    let terminal = session.terminal();
    let grid = terminal.screen().grid();
    let selected = |row: usize, col: usize| grid.is_selected(row, col);
    let paint = TerminalPaint {
        screen: terminal.screen(),
        origin: Point::new(pane.terminal.x, pane.terminal.y),
        focused: pane.focused && view.window_focused,
        selected: &selected,
    };
    scene.push_clip(pane.body);
    draw_terminal(scene, text, theme, &paint);
    scene.pop_clip();
}
