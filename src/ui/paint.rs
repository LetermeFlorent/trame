use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::files::IconStore;
use crate::geometry::Point;
use crate::render::Scene;
use crate::session::{Session, SessionId};
use crate::text::TextSystem;
use crate::theme::Theme;
use crate::ui::chrome::{self, ChromeInput, HoverPreviewFrame, LoaderView, PictureFrame};
use crate::ui::drop::{DragView, draw_drag};
use crate::ui::frame::{FrameLayout, PaneFrame};
use crate::ui::hit::Hit;
use crate::ui::motion::Motion;
use crate::ui::terminal::links::LinkCache;
use crate::ui::terminal::{ScreenCopy, TerminalPaint, draw_terminal, drawn_rows};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameView {
    pub hover: Option<Hit>,
    pub pressed: Option<Hit>,
    pub window_focused: bool,
}

impl FrameView {
    pub fn focused() -> Self {
        Self {
            window_focused: true,
            ..Self::default()
        }
    }
}

pub struct FrameSources<'a> {
    pub theme: &'a Theme,
    pub frame: &'a FrameLayout,
    pub view: &'a FrameView,
    pub sessions: &'a HashMap<SessionId, Session>,
    pub motion: &'a Motion,
    pub drag: Option<&'a DragView>,
    pub loader: Option<&'a LoaderView<'a>>,
    pub icons: &'a IconStore,
    pub links: Option<&'a RefCell<LinkCache>>,
    pub cwd: Option<&'a dyn Fn(SessionId) -> Option<PathBuf>>,
    pub picture: Option<PictureFrame>,
    pub preview: Option<HoverPreviewFrame<'a>>,
}

pub fn draw_frame(scene: &mut Scene, text: &mut TextSystem, sources: &FrameSources) {
    let input = ChromeInput {
        theme: sources.theme,
        frame: sources.frame,
        view: sources.view,
        motion: sources.motion,
        icons: sources.icons,
    };
    chrome::draw_chrome(scene, text, &input);
    if let Some(links) = sources.links {
        links.borrow_mut().keep(sources.sessions);
    }
    for pane in &sources.frame.panes {
        draw_body(scene, text, pane, sources);
    }
    chrome::draw_offline(scene, text, &input);
    chrome::draw_dividers(scene, sources.theme, sources.frame, sources.view);
    draw_overlays(scene, text, &input, sources);
    if let Some(loader) = sources.loader {
        chrome::draw_loader(scene, text, &input, loader);
    }
}

impl FrameSources<'_> {
    fn session_cwd(&self, session: &Session) -> Option<PathBuf> {
        match self.cwd {
            Some(cwd) => cwd(session.id()),
            None => session.cwd(),
        }
    }
}

fn draw_overlays(
    scene: &mut Scene,
    text: &mut TextSystem,
    input: &ChromeInput,
    sources: &FrameSources,
) {
    let frame = sources.frame;
    chrome::draw_editor(scene, text, input);
    chrome::draw_settings(scene, text, input);
    chrome::draw_menu(scene, text, input);
    if !frame.blocked
        && sources.picture.is_none()
        && sources.drag.is_none()
        && frame.menu.is_none()
        && frame.editor.is_none()
        && frame.settings.is_none()
        && frame.message.is_none()
    {
        chrome::draw_hover_preview(scene, text, input, sources.preview);
    }
    chrome::draw_picture(scene, text, input, sources.picture);
    chrome::draw_message(scene, text, input);
    match sources.drag {
        Some(drag) => draw_drag(scene, text, input, drag),
        None if frame.menu.is_none() && sources.picture.is_none() => {
            chrome::draw_tooltip(scene, text, input)
        }
        None => {}
    }
}

fn draw_body(scene: &mut Scene, text: &mut TextSystem, pane: &PaneFrame, sources: &FrameSources) {
    let theme = &sources.theme.tinted(pane.tint);
    if pane.radius > 0.0 {
        let line = sources.frame.metrics.line();
        scene.border(pane.card, theme.border_strong, pane.radius, line);
    } else {
        scene.fill(pane.body, theme.terminal, 0.0);
    }
    let Some(session) = pane.active.and_then(|id| sources.sessions.get(&id)) else {
        return;
    };
    let scroll = sources.motion.scroll_offset(session.id());
    let cwd = sources.links.and_then(|_| sources.session_cwd(session));
    let mut cache = sources.links.map(RefCell::borrow_mut);
    let (screen, links) = {
        let terminal = session.terminal();
        let shown = drawn_rows(terminal.screen(), scroll);
        let links = cache.as_mut().map_or(&[][..], |cache| {
            cache.spans(session.id(), &terminal, shown, cwd.as_deref())
        });
        (
            ScreenCopy::of(terminal.screen(), scroll).with_session(session.id().0),
            links,
        )
    };
    let paint = TerminalPaint {
        screen: &screen,
        origin: Point::new(pane.terminal.x, pane.terminal.y),
        focused: pane.focused && sources.view.window_focused,
        scroll,
        selected: &|row, col| screen.is_selected(row, col),
        links,
    };
    scene.push_clip(pane.body);
    draw_terminal(scene, text, theme, &paint);
    scene.pop_clip();
}
