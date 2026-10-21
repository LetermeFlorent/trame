use winit::window::Window;

use trame::session::SessionId;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::theme::{Appearance, Theme};
use trame::ui::drop::draw_drag;
use trame::ui::metrics::{MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE};
use trame::ui::{FrameInput, FrameLayout, FrameSources, UiMetrics, draw_frame};
use trame::workspace::{Instance, Workspace};

use super::Running;

const ATLAS_ATTEMPTS: usize = 2;

pub(super) fn compute(
    text: &mut TextSystem,
    workspace: &Workspace,
    window: &Window,
    metrics: UiMetrics,
    title: &dyn Fn(SessionId) -> String,
) -> FrameLayout {
    let size = window.inner_size();
    let input = FrameInput {
        workspace,
        width: size.width as f32,
        height: size.height as f32,
        metrics,
        maximized: window.is_maximized(),
        title,
    };
    FrameLayout::compute(text, &input)
}

impl Running {
    pub(super) fn relayout(&mut self) {
        let visible = self.workspace.active_instance().map(Instance::tabs);
        self.labels = visible
            .unwrap_or_default()
            .into_iter()
            .map(|id| (id, self.sessions.label(id)))
            .collect();
        let labels = &self.labels;
        let sessions = &self.sessions;
        let title = |id: SessionId| {
            labels
                .get(&id)
                .cloned()
                .unwrap_or_else(|| sessions.label(id))
        };
        self.frame = compute(
            &mut self.text,
            &self.workspace,
            &self.window,
            self.metrics,
            &title,
        );
        self.sync_sizes();
        self.refresh_hover();
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
        for _ in 0..ATLAS_ATTEMPTS {
            self.scene.reset(size.width as f32, size.height as f32);
            let sources = FrameSources {
                sessions: self.sessions.map(),
            };
            draw_frame(
                &mut self.scene,
                &mut self.text,
                &self.theme,
                &self.frame,
                &self.view,
                &sources,
            );
            if let Some(drag) = &drag {
                draw_drag(
                    &mut self.scene,
                    &mut self.text,
                    &self.theme,
                    &self.frame,
                    drag,
                );
            }
            if !self.text.end_frame() {
                break;
            }
        }
        let clear = self.theme.chrome;
        self.graphics
            .present(&self.window, &self.scene, self.text.atlas_mut(), clear);
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
