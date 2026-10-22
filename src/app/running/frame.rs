use std::collections::HashMap;

use std::time::Instant;

use winit::window::Window;

use trame::session::SessionId;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::theme::{Appearance, Theme};
use trame::ui::metrics::{MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE};
use trame::ui::{FrameInput, FrameLayout, FrameSources, Motion, UiMetrics, draw_frame};
use trame::workspace::{Instance, Workspace};

use super::Running;
use crate::app::sessions::Sessions;

const ATLAS_ATTEMPTS: usize = 2;

pub(super) struct LayoutInput<'a> {
    pub(super) workspace: &'a Workspace,
    pub(super) window: &'a Window,
    pub(super) metrics: UiMetrics,
}

pub(super) fn compute(
    text: &mut TextSystem,
    layout: &LayoutInput,
    motion: &mut Motion,
    title: &dyn Fn(SessionId) -> String,
) -> FrameLayout {
    let size = layout.window.inner_size();
    let input = FrameInput {
        workspace: layout.workspace,
        width: size.width as f32,
        height: size.height as f32,
        metrics: layout.metrics,
        maximized: layout.window.is_maximized(),
        title,
    };
    let frame = FrameLayout::compute_with(text, &input, motion);
    motion.finish_layout();
    frame
}

fn title_of(
    workspace: &Workspace,
    labels: &HashMap<SessionId, String>,
    sessions: &Sessions,
    id: SessionId,
) -> String {
    match workspace.tab_name(id) {
        Some(name) => name.to_owned(),
        None => labels
            .get(&id)
            .cloned()
            .unwrap_or_else(|| sessions.label(id)),
    }
}

impl Running {
    pub(super) fn relayout(&mut self) {
        let visible = self.workspace.active_instance().map(Instance::tabs);
        self.labels = visible
            .unwrap_or_default()
            .into_iter()
            .map(|id| (id, self.sessions.label(id)))
            .collect();
        let title = |id: SessionId| title_of(&self.workspace, &self.labels, &self.sessions, id);
        let layout = LayoutInput {
            workspace: &self.workspace,
            window: &self.window,
            metrics: self.metrics,
        };
        self.motion.tick(Instant::now());
        self.frame = compute(&mut self.text, &layout, &mut self.motion, &title);
        self.place_editor();
        self.sync_sizes();
        self.refresh_hover();
    }

    pub(super) fn tab_title(&self, id: SessionId) -> String {
        title_of(&self.workspace, &self.labels, &self.sessions, id)
    }

    fn sync_sizes(&mut self) {
        let cell = self.text.cell_metrics();
        for pane in &self.frame.panes {
            let size = pane.grid_size(&cell);
            for tab in &pane.tabs {
                self.sessions.resize(*tab, size);
            }
        }
    }

    pub(super) fn redraw(&mut self) {
        self.sessions.clear_output();
        self.relayout();
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        let drag = self.drag_view();
        self.motion.hover(self.view.hover);
        for _ in 0..ATLAS_ATTEMPTS {
            self.scene.reset(size.width as f32, size.height as f32);
            let sources = FrameSources {
                theme: &self.theme,
                frame: &self.frame,
                view: &self.view,
                sessions: self.sessions.map(),
                motion: &self.motion,
                drag: drag.as_ref(),
            };
            draw_frame(&mut self.scene, &mut self.text, &sources);
            if !self.text.end_frame() {
                break;
            }
        }
        let clear = self.theme.chrome;
        self.graphics
            .present(&self.window, &self.scene, self.text.atlas_mut(), clear);
        if self.motion.is_busy() {
            self.window.request_redraw();
        }
    }

    pub(super) fn output(&mut self, id: SessionId) {
        self.forward_clipboard(id);
        if self.frame.panes.iter().any(|pane| pane.active == Some(id)) {
            self.window.request_redraw();
            return;
        }
        let Some(session) = self.sessions.get(id) else {
            return;
        };
        session.take_output();
        if self
            .labels
            .get(&id)
            .is_some_and(|known| *known != session.label())
        {
            self.window.request_redraw();
        }
    }

    pub(super) fn rescale(&mut self, scale: f32) {
        self.metrics = UiMetrics::new(scale);
        self.apply_font();
    }

    pub(super) fn set_font_size(&mut self, size: f32) {
        let size = size.clamp(MIN_TERMINAL_FONT_SIZE, MAX_TERMINAL_FONT_SIZE);
        if size != self.font_size {
            self.font_size = size;
            self.apply_font();
        }
    }

    fn apply_font(&mut self) {
        let size = self.font_size * self.metrics.scale;
        self.text.set_terminal_font(size, DEFAULT_LINE_HEIGHT);
        self.changed();
    }

    pub(super) fn set_appearance(&mut self, appearance: Appearance) {
        if appearance == self.theme.appearance {
            return;
        }
        self.theme = Theme::new(appearance);
        self.sessions.set_colors(self.theme.reported_colors());
        self.window.request_redraw();
    }
}
