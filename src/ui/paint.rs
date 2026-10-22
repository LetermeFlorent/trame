use std::collections::HashMap;

use crate::geometry::Point;
use crate::render::Scene;
use crate::session::{Session, SessionId};
use crate::text::TextSystem;
use crate::theme::Theme;
use crate::ui::chrome::{self, ChromeInput};
use crate::ui::frame::{FrameLayout, PaneFrame};
use crate::ui::hit::Hit;
use crate::ui::motion::Motion;
use crate::ui::terminal::{TerminalPaint, draw_terminal};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameView {
    pub hover: Option<Hit>,
    pub pressed: Option<Hit>,
    pub window_focused: bool,
}

pub struct FrameSources<'a> {
    pub sessions: &'a HashMap<SessionId, Session>,
    pub motion: &'a Motion,
}

pub fn draw_frame(
    scene: &mut Scene,
    text: &mut TextSystem,
    theme: &Theme,
    frame: &FrameLayout,
    view: &FrameView,
    sources: &FrameSources,
) {
    let input = ChromeInput {
        theme,
        frame,
        view,
        motion: sources.motion,
    };
    chrome::draw_chrome(scene, text, &input);
    for pane in &frame.panes {
        draw_body(scene, text, theme, pane, view, sources);
    }
    chrome::draw_dividers(scene, theme, frame, view);
    chrome::draw_editor(scene, text, &input);
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
    let selected = |row: isize, col: usize| grid.is_selected_in_view(row, col);
    let paint = TerminalPaint {
        screen: terminal.screen(),
        origin: Point::new(pane.terminal.x, pane.terminal.y),
        focused: pane.focused && view.window_focused,
        scroll: pane
            .active
            .map_or(0.0, |id| sources.motion.scroll_offset(id)),
        selected: &selected,
    };
    scene.push_clip(pane.body);
    draw_terminal(scene, text, theme, &paint);
    scene.pop_clip();
}
